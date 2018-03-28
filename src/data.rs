use {Interned, NodeHandle, NodeHandleRef, Vnodes};

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
        const FLOAT = 0x10;
        const BOOL = 0x20;
        const INTERNED = 0x40;
        // --

        const ALLOCATED = 0x80;
        const BOXED = 0x100;
        const SIGNED = 0x8;

        /// No flag means no value
        const VOID = 0x0;
        const NODE_BOXED = Self::NODE.bits | Self::BOXED.bits;
        const INTEGER_SIGNED = Self::INTEGER.bits | Self::SIGNED.bits;
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
    /// 4. length of 5. parameter (number of elements)
    /// 5. arguments array
    pub action: unsafe extern "C" fn(*mut RawNodeData, *mut Vnodes, Action, usize, *const RawValue)
        -> RawValue,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct RawValue {
    pub flags: Flags,
    pub value: RawValueInner,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union RawValueInner {
    pub boolean: bool,
    pub float: f64,
    pub interned: Interned,
    pub node_data: *mut RawNodeData,
    pub signed: i64,
    pub unsigned: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value<'a> {
    Bool(bool),
    Float(f64),
    Interned(Interned),
    Node(NodeHandle),
    NodeRef(NodeHandleRef<'a>),
    Signed(i64),
    Unsigned(u64),
    Void,
}

impl<'a> Value<'a> {
    pub unsafe fn from_raw<'b>(raw: RawValue) -> Value<'b>
    where
        'a: 'b,
    {
        match raw.flags {
            Flags::INTEGER => Value::Unsigned(raw.value.unsigned),
            Flags::INTEGER_SIGNED => Value::Signed(raw.value.signed),
            Flags::BOOL => Value::Bool(raw.value.boolean),
            Flags::FLOAT => Value::Float(raw.value.float),
            Flags::INTERNED => Value::Interned(raw.value.interned),
            Flags::NODE => Value::NodeRef(NodeHandleRef::from_raw(raw.value.node_data)),
            Flags::NODE_BOXED => Value::Node(NodeHandle::from_raw(raw.value.node_data)),
            Flags::VOID => Value::Void,
            flags => unimplemented!("Unimplemented flags: {:?}", flags),
        }
    }
}

impl From<Value<'static>> for RawValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Bool(boolean) => RawValue {
                flags: Flags::BOOL,
                value: RawValueInner { boolean },
            },
            Value::Float(float) => RawValue {
                flags: Flags::FLOAT,
                value: RawValueInner { float },
            },
            Value::Interned(interned) => RawValue {
                flags: Flags::INTERNED,
                value: RawValueInner { interned },
            },
            Value::Node(node) => RawValue {
                flags: Flags::NODE | Flags::ALLOCATED,
                value: RawValueInner {
                    node_data: node.raw(),
                },
            },
            Value::NodeRef(node) => RawValue {
                flags: Flags::NODE,
                value: RawValueInner {
                    node_data: node.raw(),
                },
            },
            Value::Signed(signed) => RawValue {
                flags: Flags::INTEGER | Flags::SIGNED,
                value: RawValueInner { signed },
            },
            Value::Unsigned(unsigned) => RawValue {
                flags: Flags::INTEGER,
                value: RawValueInner { unsigned },
            },
            Value::Void => RawValue {
                flags: Flags::empty(),
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
