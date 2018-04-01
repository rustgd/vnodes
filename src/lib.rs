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

pub use alloc::{get, get_mut, pop, pop_unsized, push};
pub use conv::ValueConv;
pub use data::Value;
pub use error::{Error, Result};
pub use intern::{Interned, InternedPath, InternedPathBuf};
pub use map::MapNode;
pub use node::{Node, NodeData, NodeHandle, NodeHandleRef, NodeMut};

mod macros;

pub mod raw;

mod alloc;
mod conv;
mod data;
mod error;
mod fs;
mod intern;
mod map;
mod node;
mod util;

pub struct Vnodes {
    current: NodeHandle,
    root: NodeHandle,
}

impl Vnodes {
    pub fn new() -> Self {
        let node = MapNode::new_node();

        Vnodes {
            current: node.clone(),
            root: node,
        }
    }

    pub fn get<I, R>(&self, path: I) -> Result<R>
    where
        I: Into<InternedPathBuf>,
        R: ValueConv<'static>,
    {
        self.get_no_alloc(path, |val| R::from_value(val.make_owned()))
    }

    pub fn get_no_alloc<F, I, R>(&self, path: I, f: F) -> Result<R>
    where
        F: FnOnce(Value) -> Result<R>,
        I: Into<InternedPathBuf>,
    {
        let path_buf = path.into();
        let mut path = path_buf.path();
        let start = match path.get(0).cloned() {
            Some(Interned(0)) => {
                path = &path[1..];

                self.root.handle_ref()
            }
            _ => self.current.handle_ref(),
        };

        match path.len() {
            0 => f(ValueConv::into_value(start)),
            _ => walk_node(self, start, path, f),
        }
    }

    pub fn insert<I, V>(&self, path: I, value: V) -> Result<()>
    where
        I: Into<InternedPathBuf>,
        V: ValueConv<'static>,
    {
        let mut path_buf = path.into();
        let target = path_buf.pop().ok_or(Error::PathEmpty)?;

        let value = value.into_value();

        self.get_no_alloc(path_buf, |node| {
            node.as_node_handle()
                .map(move |node| node.insert(self, target, value))
        })
    }
}

fn walk_node<F, R>(context: &Vnodes, handle: NodeHandleRef, path: &InternedPath, f: F) -> Result<R>
where
    F: FnOnce(Value) -> Result<R>,
{
    // TODO: let node methods return Result
    // TODO: and: should the Rust `Value` even store errors?
    let value = handle.get(context, path[0]).into_res()?;

    match path.get(1).cloned() {
        None => f(value),
        Some(Interned(0)) => {
            match value {
                Value::Node(_) | Value::NodeRef(_) => {}
                _ => return Err(Error::ExpectedNode),
            }

            f(value)
        }
        Some(_) => {
            let handle_ref = match value {
                Value::Node(ref node) => node.handle_ref(),
                Value::NodeRef(node) => node,
                _ => return Err(Error::WrongType),
            };

            walk_node(context, handle_ref, &path[1..], f)
        }
    }
}
