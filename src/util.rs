use std::mem::{size_of, size_of_val, transmute};
use std::ptr::drop_in_place;
use std::slice::from_raw_parts;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct FatPtr {
    pub ptr: *mut (),
    pub size: usize,
}

impl FatPtr {
    pub unsafe fn into_fat<T: ?Sized>(self) -> *mut T {
        *(&self as *const _ as *const *mut T)
    }

    pub unsafe fn from_fat<T: ?Sized>(ptr: *mut T) -> Self {
        assert!(is_fat::<T>(), "not a fat pointer");

        *(&ptr as *const _ as *const Self)
    }
}

pub unsafe fn drop_maybe_sized<T: ?Sized>(ptr: *mut (), size: usize) {
    drop_in_place::<T>(FatPtr { ptr, size }.into_fat::<T>())
}

pub fn is_fat<T: ?Sized>() -> bool {
    size_of::<&T>() != size_of::<usize>()
}

pub unsafe fn slice_from_raw<'a, T>(raw: *const T, len: u32) -> &'a [T] {
    from_raw_parts(raw, len as usize)
}

pub unsafe fn boxed_slice_from_raw<'a, T>(raw: *const T, len: u32) -> Box<[T]> {
    let slice = slice_from_raw(raw, len);

    Box::from_raw(slice as *const [T] as *mut [T])
}
