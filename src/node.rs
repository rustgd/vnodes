use std::sync::Arc;

use any::Any;
use stack::Stack;

pub trait Node: Any {
    fn clone(&self, stack: &mut Stack);
    fn call(&self, stack: &mut Stack);
    fn get(&self, stack: &mut Stack);
    fn create(&self, stack: &mut Stack);
    fn get_or_create(&self, stack: &mut Stack);
    fn set(&self, stack: &mut Stack);
}

#[repr(C)]
pub struct NodePtr {
    node: *const Node,
}

impl NodePtr {
    pub fn from_arc(node: Arc<Node>) -> Self {
        NodePtr {
            node: Arc::into_raw(node),
        }
    }

    pub fn into_arc(self) -> Arc<Node> {
        unsafe { Arc::from_raw(self.node) }
    }
}

mod tests {
    //use super::*;
    //use std::mem::size_of;
    //use std::mem::transmute;
}
