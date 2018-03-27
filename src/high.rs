use std::ptr::{null, null_mut};

use data::{Action, Flags, NodeData as RawNodeData, Value as RawValue, ValueInner};
use {Context, Interned};

pub trait Node {
    fn call(&self, context: &Context, args: &[Value]) -> Value;

    fn get(&self, context: &Context, ident: Interned) -> Value;
}

#[repr(C)]
pub struct NodeData<T> {
    pub _raw: RawNodeData,
    pub node: T,
}

impl<T> NodeData<T>
where
    T: Node,
{
    pub fn new(node: T) -> Self {
        NodeData {
            _raw: RawNodeData {
                action: _action_node_data::<T>,
            },
            node,
        }
    }
}

unsafe extern "C" fn _action_node_data<T>(
    node_data: *const RawNodeData,
    context: *mut Context,
    action: Action,
    len: usize,
    args: *const RawValue,
) -> RawValue
where
    T: Node,
{
    let this = node_data as *const NodeData<T>;
    let this = &*this;
    let context = &*context;

    match action {
        Action::Call => unimplemented!(),
        Action::Get => {
            assert_eq!(len, 1);

            let ident = *args;
            assert_eq!(ident.flags, Flags::INTERNED);

            let ident = ident.value.interned;

            this.node.get(context, ident).into()
        }
        _ => unimplemented!(),
    }
}

pub struct NodeHandle(pub *const RawNodeData);

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
        }
    }
}
