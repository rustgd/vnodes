use {Interned, Vnodes};

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
        const SIGNED = 0x8;
        const FLOAT = 0x10;
        const BOOL = 0x20;
        const INTERNED = 0x40;
        // --

        const ALLOCATED = 0x80;
        const BOXED = 0x100;

        /// No flag means no value
        const VOID = 0x0;
        const NODE_BOXED = Self::NODE.bits | Self::BOXED.bits;
    }
}

#[repr(C)]
pub struct NodeData {
    /// A function pointer to the `action` function of this node.
    ///
    /// ## Parameters
    ///
    /// 1. self pointer
    /// 2. pointer to the `vnodes` context
    /// 3. the requested action
    /// 4. length of 5. parameter (number of elements)
    /// 5. arguments array
    pub action:
        unsafe extern "C" fn(*mut NodeData, *mut Vnodes, Action, usize, *const Value) -> Value,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Value {
    pub flags: Flags,
    pub value: ValueInner,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union ValueInner {
    pub boolean: bool,
    pub interned: Interned,
    pub node_data: *mut NodeData,
    pub signed: i64,
    pub unsigned: u64,
}
