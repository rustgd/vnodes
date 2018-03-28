#![allow(dead_code)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate derivative;
extern crate fxhash;
#[macro_use]
extern crate log;
extern crate parking_lot;

pub use alloc::Allocator;
pub use data::Value;
pub use intern::Interned;
pub use map::MapNode;
pub use node::{Node, NodeData, NodeHandle, NodeHandleRef, NodeMut};

mod macros;

pub mod raw;

mod alloc;
mod data;
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

    pub fn get(&self, ident: Interned) -> Value {
        self.root.get(self, ident)
    }

    pub fn insert(&self, ident: Interned, value: Value<'static>) {
        self.root.insert(self, ident, value);
    }
}
