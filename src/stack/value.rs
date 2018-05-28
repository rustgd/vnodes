use std::sync::Arc;

use {Any, Interned, Node, Push, Pop, Stack};

pub enum Value {
    Array(Vec<Value>),
    Bool(bool),
    Custom(Box<Any>),
    Float(f64),
    Int(i64),
    Interned(Interned),
    Node(Arc<Node>),
    Plain,
    String(String),
    Tuple(Vec<Value>),
    Uint(u64),
    Void,
}

impl Push for Value {
    fn push_tags(stack: &mut Stack) {

    }

    fn push_value(stack: &mut Stack, value: Self) {
        unimplemented!()
    }
}
