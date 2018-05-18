use stack::Stack;

#[repr(C)]
pub struct NodeRaw {
    pub raw: [u8; 32],
}

pub type NodeData = [u8; 24];

#[repr(packed)]
pub struct NodeFields {
    data: NodeData,
    vtable: *mut (),
}

pub trait NodeInterface {
    fn clone(&self, stack: &mut Stack);
    fn call(&self, stack: &mut Stack);
    fn get(&self, stack: &mut Stack);
    fn create(&self, stack: &mut Stack);
    fn get_or_create(&self, stack: &mut Stack);
    fn set(&self, stack: &mut Stack);
    fn drop(&self, stack: &mut Stack);
}

#[repr(C)]
pub struct NodeInterfaceRaw {
    pub clone: extern "C" fn(*const NodeRaw, stack: &mut Stack),
    pub call: extern "C" fn(*const NodeRaw, stack: &mut Stack),
    pub get: extern "C" fn(*const NodeRaw, stack: &mut Stack),
    pub create: extern "C" fn(*const NodeRaw, stack: &mut Stack),
    pub get_or_create: extern "C" fn(*const NodeRaw, stack: &mut Stack),
    pub set: extern "C" fn(*const NodeRaw, stack: &mut Stack),
    pub drop: extern "C" fn(*const NodeRaw),
}

mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn check_correct_size() {
        assert_eq!(size_of::<NodeFields>(), size_of::<Node>());
        assert_eq!(size_of::<Node>(), 32);
    }
}
