use std::process::abort;
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicUsize, Ordering};

use data::{Action, Flags, NodeData as RawNodeData, Value as RawValue, ValueInner};
use {Interned, Vnodes};

pub trait Node {
    fn call(&self, context: &Vnodes, args: &[Value]) -> Value;

    fn get(&self, context: &Vnodes, ident: Interned) -> Value;
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
    node_data: *const RawNodeData,
    context: *mut Vnodes,
    action: Action,
    len: usize,
    args: *const RawValue,
) -> RawValue
where
    T: Node + 'static,
{
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
                }
                _ => {}
            }

            let boxed = Box::from_raw(this as *const NodeData<T> as *mut Box<NodeData<T>>);
            drop(boxed);

            ().into()
        }
        _ => unimplemented!(),
    }
}

pub struct NodeHandle(pub *const RawNodeData);

impl NodeHandle {
    pub fn new<N>(node: N) -> Self
    where
        N: Node + 'static,
    {
        let boxed = NodeData::new(node);

        NodeHandle(Box::into_raw(boxed) as *const _)
    }
}

impl Clone for NodeHandle {
    fn clone(&self) -> Self {
        unsafe {
            let raw = &*self.0;
            (raw.action)(self.0, null_mut(), Action::Clone, 0, null());

            NodeHandle(self.0)
        }
    }
}

impl Drop for NodeHandle {
    fn drop(&mut self) {
        unsafe {
            let raw = &*self.0;
            (raw.action)(self.0, null_mut(), Action::Drop, 0, null());
        }
    }
}

unsafe impl Send for NodeHandle {}
unsafe impl Sync for NodeHandle {}

#[derive(Copy, Clone)]
pub enum Value<'a> {
    Node(&'a NodeHandle),
    Signed(i64),
    Unsigned(u64),
    Void,
}

impl From<Value<'static>> for RawValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Node(node) => RawValue {
                flags: Flags::NODE,
                value: ValueInner { node_data: node.0 },
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
