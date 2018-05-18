pub use self::data::NodeData;
pub use self::ptr::{NodeHandle, NodeHandleRef};

use parking_lot::RwLock;

use raw::*;
use {Interned, Result, Value, ValueConv, Vnodes};

mod data;
mod ptr;

pub trait Node: Send + Sync {
    fn call(&self, context: &Vnodes, args: &[Value]) -> Result<Value>;

    fn get(&self, context: &Vnodes, ident: Interned) -> Result<Value>;

    fn set(&self, context: &Vnodes, ident: Interned, value: Value<'static>) -> Result<()>;
}

pub trait NodeMut: Send + Sync {
    fn call(&self, context: &Vnodes, args: &[Value]) -> Result<Value<'static>>;

    fn get(&self, context: &Vnodes, ident: Interned) -> Result<Value<'static>>;

    fn set(&mut self, context: &Vnodes, ident: Interned, value: Value<'static>) -> Result<()>;
}

impl<T> Node for RwLock<T>
where
    T: NodeMut,
{
    fn call(&self, context: &Vnodes, args: &[Value]) -> Result<Value> {
        self.read().call(context, args)
    }

    fn get(&self, context: &Vnodes, ident: Interned) -> Result<Value> {
        self.read().get(context, ident)
    }

    fn set(&self, context: &Vnodes, ident: Interned, value: Value<'static>) -> Result<()> {
        self.write().set(context, ident, value)
    }
}
