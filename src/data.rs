#[repr(u8)]
pub enum Action {
    Call = 0x0,
    Get = 0x1,
    Set = 0x2,
}

pub struct Context {}

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
        // --

        /// Set if the node accepts idents in interned form.
        const INTERNED = 0x2;
        /// Set if it's a pointer (not a value).
        /// Not used for nodes.
        const PTR = 0x80;

        // -- Convenience flags
        const UNINTERNED_NODE = Self::NODE.bits;
        const INTERNED_NODE = Self::NODE.bits | Self::INTERNED.bits;
        // --
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InternedSingle(pub u64);

#[repr(C)]
#[derive(Copy, Clone)]
pub struct NodeData {
    pub action:
        unsafe extern "C" fn(*mut (), *mut Context, Action, usize, *const [u8]) -> Value,
}

#[repr(C)]
pub struct Value {
    pub flags: Flags,
    pub value: ValueInner,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union ValueInner {
    pub boolean: bool,
    pub boolean_ptr: *mut bool,
    pub node_data: *mut NodeData,
    pub signed: i64,
    pub signed_ptr: *mut i64,
    pub unsigned: u64,
    pub unsigned_ptr: *mut u64,
}
