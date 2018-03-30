use std::path::PathBuf;

use {Interned, Node, NodeHandle, Result, Value, Vnodes};

pub struct FsNode {
    path: PathBuf,
}

impl FsNode {
    pub fn new_node<P>(path: P) -> NodeHandle
    where
        P: Into<PathBuf>,
    {
        NodeHandle::new(FsNode { path: path.into() })
    }
}

impl Node for FsNode {
    fn call(&self, context: &Vnodes, args: &[Value]) -> Result<Value> {
        unimplemented!()
    }

    fn get(&self, context: &Vnodes, ident: Interned) -> Result<Value> {
        unimplemented!()
    }

    fn set(&self, context: &Vnodes, ident: Interned, value: Value) -> Result<()> {
        unimplemented!()
    }
}
