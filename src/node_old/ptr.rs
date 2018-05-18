use std::marker::PhantomData;
use std::mem::forget;
use std::ptr::null_mut;

use super::*;

#[derive(Debug, PartialEq)]
pub struct NodeHandle {
    data: NodeHandleRef<'static>,
}

impl NodeHandle {
    pub fn new<N>(node: N) -> Self
    where
        N: Node + 'static,
    {
        let boxed = NodeData::new(node);

        trace!(
            "node data with node {:x} created",
            boxed.as_ref() as *const _ as usize
        );

        unsafe { NodeHandle::from_raw(Box::into_raw(boxed) as *mut RawNodeData) }
    }

    pub unsafe fn from_raw(inner: *mut RawNodeData) -> Self {
        NodeHandle {
            data: NodeHandleRef::from_raw(inner),
        }
    }

    pub fn get(&self, context: &Vnodes, ident: Interned) -> Value {
        self.data.get(context, ident)
    }

    pub fn insert(&self, context: &Vnodes, ident: Interned, value: Value<'static>) {
        self.data.insert(context, ident, value);
    }

    pub fn into_raw(this: Self) -> *mut RawNodeData {
        let raw = this.data.raw();

        forget(this);

        raw
    }

    pub fn as_raw(&self) -> *mut RawNodeData {
        self.data.raw()
    }

    pub fn handle_ref<'a>(&'a self) -> NodeHandleRef<'a> {
        unsafe { NodeHandleRef::from_raw(self.as_raw()) }
    }
}

impl Clone for NodeHandle {
    fn clone(&self) -> Self {
        self.data.to_handle()
    }
}

impl Drop for NodeHandle {
    fn drop(&mut self) {
        unsafe {
            NodeHandleRef::drop(&mut self.data);
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct NodeHandleRef<'a> {
    inner: *mut RawNodeData,
    marker: PhantomData<&'a Node>,
}

impl<'a> NodeHandleRef<'a> {
    pub unsafe fn from_raw(inner: *mut RawNodeData) -> Self {
        NodeHandleRef {
            inner,
            marker: PhantomData,
        }
    }

    pub fn get<'b>(&'b self, context: &Vnodes, ident: Interned) -> Value<'b> {
        let ident: RawValue = Value::Interned(ident).into();

        unsafe {
            Self::action(
                self,
                context as *const Vnodes as RawContextPtr,
                Action::Get,
                ident,
            )
        }
    }

    pub fn insert(&self, context: &Vnodes, ident: Interned, value: Value<'static>) {
        unsafe {
            Self::action(
                self,
                context as *const Vnodes as RawContextPtr,
                Action::Set,
                (ident, value).into_value().into(),
            );
        }
    }

    pub fn raw(&self) -> *mut RawNodeData {
        self.inner
    }

    pub fn to_handle(&self) -> NodeHandle {
        unsafe {
            Self::clone(self);

            NodeHandle::from_raw(self.inner)
        }
    }

    pub unsafe fn action<'b>(
        this: &'b Self,
        context: *mut Vnodes,
        action: Action,
        arg: RawValue,
    ) -> Value<'b> {
        let raw = &*this.inner;

        Value::from_raw((raw.action)(this.inner, context, action, arg))
    }

    pub unsafe fn clone(this: &Self) {
        Self::action(this, null_mut(), Action::Clone, Value::Void.into());
    }

    pub unsafe fn drop(this: &mut Self) {
        Self::action(this, null_mut(), Action::Drop, Value::Void.into());
    }
}

unsafe impl<'a> Send for NodeHandleRef<'a> {}
unsafe impl<'a> Sync for NodeHandleRef<'a> {}
