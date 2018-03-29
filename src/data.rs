use std::fmt::{Debug, Formatter, Result as FmtResult};

use {Error, Interned, InternedPath, InternedPathBuf, NodeHandle, NodeHandleRef, Result, Vnodes};

#[repr(u8)]
#[derive(Debug)]
pub enum Action {
    Call = 0x0,
    List = 0x1,
    Get = 0x10,
    Set = 0x12,
    Clone = 0x20,
    Drop = 0x21,
}

bitflags! {
    #[repr(C)]
    pub struct Flags: u32 {
        // -- Exclusive flags
        const NODE = 0x1;
        const STRING = 0x2;
        const INTEGER = 0x4;
        const FLOAT = 0x8;
        const BOOL = 0x10;
        const INTERNED = 0x20;
        const ERROR = 0x40;
        // --

        const SIGNED = 0x100;
        const ALLOCATED = 0x200;
        const BOXED = 0x400;
        const ARRAY = 0x800;

        /// No flag means no value
        const VOID = 0x0;
        const VALUE_ARRAY = Self::ARRAY.bits;
        const VALUE_ARRAY_BOXED = Self::ARRAY.bits | Self::BOXED.bits;
        const NODE_BOXED = Self::NODE.bits | Self::BOXED.bits;
        const INTEGER_SIGNED = Self::INTEGER.bits | Self::SIGNED.bits;
        const INTERNED_PATH = Self::INTERNED.bits | Self::ARRAY.bits;
        const INTERNED_PATH_BUF = Self::INTERNED.bits | Self::ARRAY.bits | Self::BOXED.bits;
    }
}

#[repr(C)]
pub struct RawNodeData {
    /// A function pointer to the `action` function of this node.
    ///
    /// ## Parameters
    ///
    /// 1. self pointer
    /// 2. pointer to the `vnodes` context
    /// 3. the requested action
    /// 4. argument(s)
    pub action: unsafe extern "C" fn(*mut RawNodeData, *mut Vnodes, Action, RawValue) -> RawValue,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct RawValue {
    pub flags: Flags,
    pub extra: u32,
    pub value: RawValueInner,
}

impl Debug for RawValue {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        unsafe {
            let value = Value::from_raw(*self);
            f.debug_struct("RawValue")
                .field("value_repr", &value)
                .finish()
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union RawValueInner {
    pub boolean: bool,
    pub error: Error,
    pub float: f64,
    pub interned: Interned,
    pub interned_path: *const Interned,
    pub node_data: *mut RawNodeData,
    pub signed: i64,
    pub unsigned: u64,
    pub value_array: *const RawValue,
}

unsafe impl Send for RawValueInner {}
unsafe impl Sync for RawValueInner {}

#[derive(Clone, Debug)]
pub enum Value<'a> {
    Bool(bool),
    Error(Error),
    Float(f64),
    Interned(Interned),
    InternedPathBuf(InternedPathBuf),
    InternedPathRef(&'a InternedPath),
    Node(NodeHandle),
    NodeRef(NodeHandleRef<'a>),
    Signed(i64),
    Unsigned(u64),
    ValueArray(Box<[RawValue]>),
    ValueArrayRef(&'a [RawValue]),
    Void,
}

impl<'a> Value<'a> {
    pub unsafe fn from_raw<'b>(raw: RawValue) -> Value<'b>
    where
        'a: 'b,
    {
        use std::slice::from_raw_parts;

        match raw.flags {
            Flags::BOOL => Value::Bool(raw.value.boolean),
            Flags::ERROR => Value::Error(raw.value.error),
            Flags::FLOAT => Value::Float(raw.value.float),
            Flags::INTEGER => Value::Unsigned(raw.value.unsigned),
            Flags::INTEGER_SIGNED => Value::Signed(raw.value.signed),
            Flags::INTERNED => Value::Interned(raw.value.interned),
            Flags::INTERNED_PATH => {
                Value::InternedPathRef(from_raw_parts(raw.value.interned_path, raw.extra as usize))
            }
            Flags::INTERNED_PATH_BUF => {
                let slice = from_raw_parts(raw.value.interned_path, raw.extra as usize);
                let boxed = Box::from_raw(slice as *const [Interned] as *mut [Interned]);

                Value::InternedPathBuf(InternedPathBuf::from(boxed))
            }
            Flags::NODE => Value::NodeRef(NodeHandleRef::from_raw(raw.value.node_data)),
            Flags::NODE_BOXED => Value::Node(NodeHandle::from_raw(raw.value.node_data)),
            Flags::VALUE_ARRAY => {
                Value::ValueArrayRef(from_raw_parts(raw.value.value_array, raw.extra as usize))
            }
            Flags::VALUE_ARRAY_BOXED => {
                let slice = from_raw_parts(raw.value.value_array, raw.extra as usize);

                Value::ValueArray(Box::from_raw(slice as *const [RawValue] as *mut [RawValue]))
            }
            Flags::VOID => Value::Void,
            flags => unimplemented!("Unimplemented flags: {:?}", flags),
        }
    }

    pub fn from_res(res: Result<Self>) -> Self {
        match res {
            Ok(x) => x,
            Err(e) => Value::Error(e),
        }
    }

    pub fn into_res(self) -> Result<Self> {
        match self {
            Value::Error(e) => Err(e),
            x => Ok(x),
        }
    }

    pub fn make_owned(self) -> Value<'static> {
        match self {
            Value::Bool(b) => Value::Bool(b),
            Value::Error(e) => Value::Error(e),
            Value::Float(f) => Value::Float(f),
            Value::Interned(i) => Value::Interned(i),
            Value::InternedPathBuf(b) => Value::InternedPathBuf(b),
            Value::InternedPathRef(r) => Value::InternedPathBuf(InternedPathBuf::from(r)),
            Value::Node(n) => Value::Node(n),
            Value::NodeRef(n) => Value::Node(n.to_handle()),
            Value::Signed(s) => Value::Signed(s),
            Value::Unsigned(u) => Value::Unsigned(u),
            Value::ValueArray(a) => Value::ValueArray(a),
            Value::ValueArrayRef(a) => Value::ValueArray(Vec::from(a).into_boxed_slice()),
            Value::Void => Value::Void,
        }
    }
}

impl<'a> From<()> for Value<'a> {
    fn from(_: ()) -> Self {
        Value::Void
    }
}

impl<'a> From<Value<'a>> for RawValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Bool(boolean) => RawValue {
                flags: Flags::BOOL,
                extra: 0,
                value: RawValueInner { boolean },
            },
            Value::Error(error) => RawValue {
                flags: Flags::ERROR,
                extra: 0,
                value: RawValueInner { error },
            },
            Value::Float(float) => RawValue {
                flags: Flags::FLOAT,
                extra: 0,
                value: RawValueInner { float },
            },
            Value::Interned(interned) => RawValue {
                flags: Flags::INTERNED,
                extra: 0,
                value: RawValueInner { interned },
            },
            Value::InternedPathBuf(b) => {
                let boxed = b.into_boxed_slice();
                RawValue {
                    flags: Flags::INTERNED_PATH_BUF,
                    extra: boxed.len() as u32,
                    value: RawValueInner {
                        interned_path: Box::into_raw(boxed) as *const Interned,
                    },
                }
            }
            Value::InternedPathRef(r) => {
                let ptr = r.as_ptr();
                RawValue {
                    flags: Flags::INTERNED_PATH,
                    extra: r.len() as u32,
                    value: RawValueInner { interned_path: ptr },
                }
            }
            Value::Node(node) => RawValue {
                flags: Flags::NODE | Flags::ALLOCATED,
                extra: 0,
                value: RawValueInner {
                    node_data: node.raw(),
                },
            },
            Value::NodeRef(node) => RawValue {
                flags: Flags::NODE,
                extra: 0,
                value: RawValueInner {
                    node_data: node.raw(),
                },
            },
            Value::Signed(signed) => RawValue {
                flags: Flags::INTEGER | Flags::SIGNED,
                extra: 0,
                value: RawValueInner { signed },
            },
            Value::Unsigned(unsigned) => RawValue {
                flags: Flags::INTEGER,
                extra: 0,
                value: RawValueInner { unsigned },
            },
            Value::ValueArray(array) => RawValue {
                flags: Flags::VALUE_ARRAY_BOXED,
                extra: array.len() as u32,
                value: RawValueInner {
                    value_array: Box::into_raw(array) as *const RawValue,
                },
            },
            Value::ValueArrayRef(array) => RawValue {
                flags: Flags::VALUE_ARRAY,
                extra: array.len() as u32,
                value: RawValueInner {
                    value_array: array.as_ptr(),
                },
            },
            Value::Void => RawValue {
                flags: Flags::empty(),
                extra: 0,
                value: RawValueInner { unsigned: 0x0 },
            },
        }
    }
}

impl From<()> for RawValue {
    fn from(_: ()) -> Self {
        Value::Void.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn check_size() {
        assert_eq!(size_of::<Flags>(), 4);
        assert_eq!(size_of::<u32>(), 4);
        assert_eq!(size_of::<RawValueInner>(), 8);
        assert_eq!(size_of::<RawValue>(), 16);
    }
}
