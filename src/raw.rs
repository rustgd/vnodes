#![no_mangle]

pub use data::{Action, Flags, RawNodeData, RawValue, RawValueInner};
pub use {Interned, Vnodes};

pub use self::export::*;

pub type RawContextPtr = *mut Vnodes;
pub type RawNodePtr = *mut RawNodeData;
pub type RawValueList = *const RawValue;

mod export {
    use super::*;
    use Value;

    pub unsafe extern "C" fn vnodes_context_insert(
        context: RawContextPtr,
        ident: Interned,
        value: RawValue,
    ) {
        let context = &*context;

        context.insert(ident, Value::from_raw(value));
    }
}
