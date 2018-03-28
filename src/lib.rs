#![allow(dead_code)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate failure;
extern crate fxhash;
#[macro_use]
extern crate log;
extern crate parking_lot;

pub use alloc::Allocator;
pub use conv::ValueConv;
pub use data::Value;
pub use error::{Error, Result};
pub use intern::Interned;
pub use map::MapNode;
pub use node::{Node, NodeData, NodeHandle, NodeHandleRef, NodeMut};

mod macros;

pub mod raw;

mod alloc;
mod conv;
mod data;
mod error;
mod intern;
mod map;
mod node;

pub struct Vnodes {
    allocator: Allocator,
    current: NodeHandle,
    root: NodeHandle,
}

impl Vnodes {
    pub fn new() -> Self {
        let node = MapNode::new_node();

        Vnodes {
            allocator: Allocator::new(),
            current: node.clone(),
            root: node,
        }
    }

    pub fn get<'a, I, R>(&'a self, ident: I) -> Result<R>
    where
        I: Into<Interned>,
        R: ValueConv<'a>,
    {
        R::from_value(self.root.get(self, ident.into()))
    }

    pub fn insert<I, V>(&self, ident: I, value: V)
    where
        I: Into<Interned>,
        V: ValueConv<'static>,
    {
        self.root.insert(self, ident.into(), value.into_value());
    }
}
