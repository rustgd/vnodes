#![no_mangle]
#![allow(unused_variables)]

pub use data::{Action, Flags, RawNodeData, RawValue, RawValueInner};
pub use {Interned, Vnodes};

pub use self::export::*;

pub type RawContextPtr = *mut Vnodes;
pub type RawNodePtr = *mut RawNodeData;
pub type RawValueList = *const RawValue;

mod export {
    use super::*;

    pub unsafe extern "C" fn vnodes_context_insert(
        context: RawContextPtr,
        ident: Interned,
        value: RawValue,
    ) {
        let context = &*context;

        // TODO
        //context.insert(&[ident], Value::from_raw(value));
        unimplemented!()
    }
}
