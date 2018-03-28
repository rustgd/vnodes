use std::marker::PhantomData;
use std::process::abort;
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicUsize, Ordering};

use parking_lot::RwLock;

use data::{Action, Flags, NodeData as RawNodeData, Value as RawValue, ValueInner};
use {Interned, Vnodes};

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
        _ => unimplemented!(),
    }
}

fn drop_box<T>(b: Box<NodeData<T>>) {
    drop(b);
}

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

#[derive(Copy, Clone)]
pub struct NodeHandleRef<'a> {
    inner: *mut RawNodeData,
    marker: PhantomData<&'a Node>,
}

impl<'a> NodeHandleRef<'a> {
    pub fn from_raw(inner: *mut RawNodeData) -> Self {
        NodeHandleRef {
            inner,
            marker: PhantomData,
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

#[derive(Clone)]
pub enum Value<'a> {
    Bool(bool),
    Float(f64),
    Interned(Interned),
    Node(NodeHandle),
    NodeRef(NodeHandleRef<'a>),
    Signed(i64),
    Unsigned(u64),
    Void,
}

impl<'a> Value<'a> {
    pub unsafe fn from_raw<'b>(raw: RawValue) -> Value<'b>
    where
        'a: 'b,
    {
        match raw.flags {
            Flags::INTEGER => Value::Unsigned(raw.value.unsigned),
            Flags::INTEGER_SIGNED => Value::Unsigned(raw.value.unsigned),
            Flags::BOOL => Value::Bool(raw.value.boolean),
            Flags::FLOAT => Value::Float(raw.value.float),
            Flags::INTERNED => Value::Interned(raw.value.interned),
            Flags::NODE => Value::NodeRef(NodeHandleRef::from_raw(raw.value.node_data)),
            Flags::NODE_BOXED => Value::Node(NodeHandle::from_raw(raw.value.node_data)),
            Flags::VOID => Value::Void,
            flags => unimplemented!("Unimplemented flags: {:?}", flags),
        }
    }
}

impl From<Value<'static>> for RawValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Bool(boolean) => RawValue {
                flags: Flags::BOOL,
                value: ValueInner { boolean },
            },
            Value::Float(float) => RawValue {
                flags: Flags::FLOAT,
                value: ValueInner { float },
            },
            Value::Interned(interned) => RawValue {
                flags: Flags::INTERNED,
                value: ValueInner { interned },
            },
            Value::Node(node) => RawValue {
                flags: Flags::NODE | Flags::ALLOCATED,
                value: ValueInner {
                    node_data: node.raw(),
                },
            },
            Value::NodeRef(node) => RawValue {
                flags: Flags::NODE,
                value: ValueInner {
                    node_data: node.raw(),
                },
            },
            Value::Signed(signed) => RawValue {
                flags: Flags::INTEGER | Flags::SIGNED,
                value: ValueInner { signed },
            },
            Value::Unsigned(unsigned) => RawValue {
                flags: Flags::INTEGER,
                value: ValueInner { unsigned },
            },
            Value::Void => RawValue {
                flags: Flags::empty(),
                value: ValueInner { unsigned: 0x0 },
            },
        }
    }
}

impl From<()> for RawValue {
    fn from(_: ()) -> Self {
        Value::Void.into()
    }
}
