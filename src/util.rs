use std::slice::from_raw_parts;

pub unsafe fn slice_from_raw<'a, T>(raw: *const T, len: u32) -> &'a [T] {
    from_raw_parts(raw, len as usize)
}

pub unsafe fn boxed_slice_from_raw<'a, T>(raw: *const T, len: u32) -> Box<[T]> {
    let slice = slice_from_raw(raw, len);

    Box::from_raw(slice as *const [T] as *mut [T])
}
