#![allow(dead_code)]

#[macro_use]
extern crate bitflags;
extern crate fxhash;

pub use alloc::Allocator;
pub use high::{Node, NodeData, NodeHandle, Value};
pub use intern::Interned;

mod macros;

mod alloc;
mod data;
mod high;
mod intern;
mod map;

pub struct Context {
    allocator: Allocator,
    current: NodeHandle,
    root: NodeHandle,
}

impl Context {
    pub fn new() -> Self {
        Context {
            allocator: Allocator::new(),
            current: (),
            root: (),
        }
    }
}
