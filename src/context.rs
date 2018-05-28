use std::sync::Arc;

use node::Node;

pub struct Context {
    root: Arc<Node>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            root: unimplemented!(),
        }
    }
}
