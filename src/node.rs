use std::marker::PhantomData;
use std::process::abort;
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicUsize, Ordering};

use parking_lot::RwLock;

use raw::*;
use {Interned, Value, Vnodes};

pub trait Node: Send + Sync {
    fn call(&self, context: &Vnodes, args: &[Value]) -> Value;

    fn get(&self, context: &Vnodes, ident: Interned) -> Value;

    fn set(&self, context: &Vnodes, ident: Interned, value: Value<'static>);
}

pub trait NodeMut: Send + Sync {
    fn call(&self, context: &Vnodes, args: &[Value]) -> Value<'static>;

    fn get(&self, context: &Vnodes, ident: Interned) -> Value<'static>;

    fn set(&mut self, context: &Vnodes, ident: Interned, value: Value<'static>);
}

impl<T> Node for RwLock<T>
where
    T: NodeMut,
{
    fn call(&self, context: &Vnodes, args: &[Value]) -> Value {
        self.read().call(context, args)
    }

    fn get(&self, context: &Vnodes, ident: Interned) -> Value {
        self.read().get(context, ident)
    }

    fn set(&self, context: &Vnodes, ident: Interned, value: Value<'static>) {
        self.write().set(context, ident, value)
    }
}

#[repr(C)]
pub struct NodeData<T> {
    pub _raw: RawNodeData,
    pub node: T,
    strong: AtomicUsize,
    //weak: AtomicUsize,
}

impl<T> NodeData<T>
where
    T: Node + 'static,
{
    fn new(node: T) -> Box<Self> {
        let data = NodeData {
            _raw: RawNodeData {
                action: _action_node_data::<T>,
            },
            node,
            strong: AtomicUsize::new(1),
            //weak: AtomicUsize::new(0),
        };

        Box::new(data)
    }
}

unsafe extern "C" fn _action_node_data<T>(
    node_data: *mut RawNodeData,
    context: *mut Vnodes,
    action: Action,
    len: usize,
    args: *const RawValue,
) -> RawValue
where
    T: Node + 'static,
{
    trace!(
        "action function called with args {:x}, {:x}, {:?}, {}, {:x}",
        node_data as usize,
        context as usize,
        action,
        len,
        args as usize,
    );

    let this = node_data as *const NodeData<T>;
    let this = &*this;

    match action {
        Action::Call => unimplemented!(),
        Action::Get => {
            let context = &*context;
            assert_eq!(len, 1);

            let ident = *args;
            assert_eq!(ident.flags, Flags::INTERNED);

            let ident = ident.value.interned;

            this.node.get(context, ident).into()
        }
        Action::Set => {
            let context = &*context;
            assert_eq!(len, 2);

            let args = args as *const [RawValue; 2];
            let args = &*args;

            let ident = args[0];
            assert_eq!(ident.flags, Flags::INTERNED);
            let ident = ident.value.interned;

            let value = args[1];
            let value = Value::from_raw(value);

            this.node.set(context, ident, value).into()
        }
        Action::Clone => {
            let old = this.strong.fetch_add(1, Ordering::Relaxed);

            // Refcount is way too high, we got serious memory leaks.
            if old > (!0 >> 1) {
                abort();
            }

            ().into()
        }
        Action::Drop => {
            let old = this.strong.fetch_sub(1, Ordering::Relaxed);

            match old {
                1 => {
                    // Synchronize with other threads to make sure we have unique ownership
                    assert_eq!(this.strong.load(Ordering::Relaxed), 0);

                    // Don't rely on type inference here
                    let b: Box<NodeData<T>> = Box::from_raw(node_data as *mut NodeData<T>);
                    drop(b);
                }
                _ => {}
            }

            ().into()
        }
        Action::List => unimplemented!(),
    }
}

fn drop_box<T>(b: Box<NodeData<T>>) {
    drop(b);
}

#[derive(Debug, PartialEq)]
pub struct NodeHandle {
    data: NodeHandleRef<'static>,
}

impl NodeHandle {
    pub fn new<N>(node: N) -> Self
    where
        N: Node + 'static,
    {
        let boxed = NodeData::new(node);

        unsafe { NodeHandle::from_raw(Box::into_raw(boxed) as *mut RawNodeData) }
    }

    pub unsafe fn from_raw(inner: *mut RawNodeData) -> Self {
        NodeHandle {
            data: NodeHandleRef::from_raw(inner),
        }
    }

    pub fn get(&self, context: &Vnodes, ident: Interned) -> Value {
        self.data.get(context, ident)
    }

    pub fn insert(&self, context: &Vnodes, ident: Interned, value: Value<'static>) {
        self.data.insert(context, ident, value);
    }

    pub fn raw(&self) -> *mut RawNodeData {
        self.data.raw()
    }
}

impl Clone for NodeHandle {
    fn clone(&self) -> Self {
        self.data.to_handle()
    }
}

impl Drop for NodeHandle {
    fn drop(&mut self) {
        unsafe {
            NodeHandleRef::drop(&mut self.data);
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct NodeHandleRef<'a> {
    inner: *mut RawNodeData,
    marker: PhantomData<&'a Node>,
}

impl<'a> NodeHandleRef<'a> {
    pub unsafe fn from_raw(inner: *mut RawNodeData) -> Self {
        NodeHandleRef {
            inner,
            marker: PhantomData,
        }
    }

    pub fn get<'b>(&'b self, context: &Vnodes, ident: Interned) -> Value<'b> {
        let ident: RawValue = Value::Interned(ident).into();

        unsafe {
            Self::action(
                self,
                context as *const Vnodes as RawContextPtr,
                Action::Get,
                1,
                &ident as RawValueList,
            )
        }
    }

    pub fn insert(&self, context: &Vnodes, ident: Interned, value: Value<'static>) {
        let ident: RawValue = Value::Interned(ident).into();
        let raw_value: RawValue = value.into();
        let list = [ident, raw_value];
        let list: RawValueList = list.as_ptr();

        unsafe {
            Self::action(
                self,
                context as *const Vnodes as RawContextPtr,
                Action::Set,
                2,
                list,
            );
        }
    }

    pub fn raw(&self) -> *mut RawNodeData {
        self.inner
    }

    pub fn to_handle(&self) -> NodeHandle {
        unsafe {
            Self::clone(self);

            NodeHandle::from_raw(self.inner)
        }
    }

    pub unsafe fn action<'b>(
        this: &'b Self,
        context: *mut Vnodes,
        action: Action,
        len: usize,
        args: *const RawValue,
    ) -> Value<'b> {
        let raw = &*this.inner;

        Value::from_raw((raw.action)(this.inner, context, action, len, args))
    }

    pub unsafe fn clone(this: &Self) {
        Self::action(this, null_mut(), Action::Clone, 0, null());
    }

    pub unsafe fn drop(this: &mut Self) {
        Self::action(this, null_mut(), Action::Drop, 0, null());
    }
}

unsafe impl<'a> Send for NodeHandleRef<'a> {}
unsafe impl<'a> Sync for NodeHandleRef<'a> {}
