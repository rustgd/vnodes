use std::marker::PhantomData;
use std::mem::{forget, size_of, size_of_val};
use std::ptr;

use util::drop_maybe_sized;

const ALIGN: usize = 0x1 << ALIGN_LOG;
const ALIGN_LOG: usize = 4;
const CAP: usize = 0x1 << 14;
const MAX_BYTES_PER_ALLOC: usize = MAX_CELLS_PER_ALLOC << ALIGN_LOG;
const MAX_CELLS_PER_ALLOC: usize = 0x1 << 8;
const NUM_CELLS: usize = CAP << ALIGN_LOG;

pub struct Allocator {
    pub raw: RawAllocator,
}

impl Allocator {
    pub fn new() -> Self {
        Allocator {
            raw: RawAllocator::new(),
        }
    }

    pub fn get<'a: 'b, 'b, T>(&'a self, index: &'b AllocIndex<T>) -> &'b T {
        unsafe { &*(self.raw.ptr(index.index as usize) as *const T) }
    }

    pub fn get_mut<'a: 'b, 'b, T>(&'a mut self, index: &'b mut AllocIndex<T>) -> &'b mut T {
        unsafe { &mut *(self.raw.ptr_mut(index.index as usize) as *mut T) }
    }

    pub fn push<T>(&mut self, value: T) -> AllocIndex<T> {
        let size = size_of::<T>();
        let index = self.raw.allocate(size_of::<T>());

        unsafe {
            ptr::write(self.raw.ptr_mut(index) as *mut T, value);
        }

        let index = index as u32;
        let size = size as u32;

        AllocIndex {
            index,
            size,
            marker: PhantomData,
        }
    }

    pub fn push_boxed<T: ?Sized>(&mut self, value: Box<T>) -> AllocIndex<T> {
        let size = size_of_val(value.as_ref());
        let index = self.raw.allocate(size);

        unsafe {
            ptr::copy_nonoverlapping(
                Box::into_raw(value) as *const T as *const u8,
                self.raw.ptr_mut(index) as *mut u8,
                size as usize,
            );
        }

        let index = index as u32;
        let size = size as u32;

        AllocIndex {
            index,
            size,
            marker: PhantomData,
        }
    }

    pub fn pop<T>(&mut self, index: AllocIndex<T>) -> T {
        unsafe {
            let ptr = self.raw.ptr_mut(index.index as usize);
            let value = ptr::read(ptr as *const T);
            self.raw
                .deallocate(index.index as usize, index.size as usize);

            // Dropping an index logs an error about a memory leak.
            // There's no actual dropping going on in `AllocIndex`,
            // so this doesn't actually leak memory.
            forget(index);

            value
        }
    }

    pub fn pop_unsized<T: ?Sized>(&mut self, index: AllocIndex<T>) {
        unsafe {
            let ptr = self.raw.ptr_mut(index.index as usize);
            drop_maybe_sized::<T>(ptr, index.size as usize);
            self.raw
                .deallocate(index.index as usize, index.size as usize);

            // Dropping an index logs an error about a memory leak.
            // There's no actual dropping going on in `AllocIndex`,
            // so this doesn't actually leak memory.
            forget(index);
        }
    }
}

#[derive(Debug)]
#[must_use]
pub struct AllocIndex<T: ?Sized> {
    index: u32,
    size: u32,
    marker: PhantomData<T>,
}

impl<T: ?Sized> Drop for AllocIndex<T> {
    fn drop(&mut self) {
        error!("Memory leak: `AllocIndex` dropped, value was not popped")
    }
}

pub struct RawAllocator {
    buffer: Box<[[u8; ALIGN]]>,
    prev_size: Vec<usize>,
    cursor: usize,
}

impl RawAllocator {
    pub fn new() -> Self {
        RawAllocator {
            buffer: vec![[0; ALIGN]; NUM_CELLS].into_boxed_slice(),
            cursor: 0,
            prev_size: vec![0],
        }
    }

    pub fn allocate(&mut self, bytes: usize) -> usize {
        let cells = aligned_cells(bytes);
        let start = self.cursor;
        let end = start + cells;

        assert!(NUM_CELLS > end, "Allocator went out of memory");

        self.cursor += cells;

        ensure_index(&mut self.prev_size, self.cursor);
        self.prev_size[self.cursor] = cells;

        start
    }

    pub unsafe fn deallocate(&mut self, index: usize, size: usize) {
        let num_cells = aligned_cells(size);
        let end = index + num_cells;

        if self.cursor == end {
            let to_dealloc = self.prev_size[self.cursor];
            self.cursor -= to_dealloc;
        } else {
            let prev_size = self.prev_size[end];
            self.prev_size[end] = 0;
            let bump_index = self.prev_size[end + 1..self.cursor + 1]
                .iter()
                .position(|&x| x != 0)
                .expect("Nothing to deallocate here");
            // Re-normalize
            let bump_index = bump_index + end + 1;

            self.prev_size[bump_index] += prev_size;
        }
    }

    pub unsafe fn ptr(&self, index: usize) -> *const () {
        &self.buffer[index] as *const _ as *const ()
    }

    pub unsafe fn ptr_mut(&mut self, index: usize) -> *mut () {
        &mut self.buffer[index] as *mut _ as *mut ()
    }

    pub fn debug(&mut self) {
        println!(
            "Allocator {{ buffer: {:#?}, prev_size: {:?}, cursor: {} }}",
            "<>",
            &self.prev_size[0..self.cursor + 1],
            self.cursor
        );
    }
}

fn aligned_cells(bytes: usize) -> usize {
    assert!(
        bytes <= MAX_BYTES_PER_ALLOC,
        "Cannot allocate that many bytes"
    );

    let mut cells = bytes >> ALIGN_LOG;

    if bytes & (ALIGN - 1) != 0 {
        cells += 1;
    }

    cells
}

fn conv64(b: &[u8]) -> u64 {
    unsafe { *(b.as_ptr() as *const u64) }
}

fn ensure_index(v: &mut Vec<usize>, index: usize) {
    use std::cmp::min;
    use std::iter::repeat;

    let len = index + 1;

    let num = len - min(v.len(), len);
    v.extend(repeat(0).take(num));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push() {
        let mut alloc = Allocator::new();
        let mut index: AllocIndex<(i32, i32, i32)> = alloc.push((5, 1, 3));

        assert_eq!(alloc.get(&index), &(5, 1, 3));

        alloc.get_mut(&mut index).1 = 13;

        assert_eq!(alloc.get(&index), &(5, 13, 3));

        alloc.pop(index);
    }

    #[test]
    fn test_wrong_pop() {
        let mut alloc = Allocator::new();
        let mut index: AllocIndex<(i32, i32, i32)> = alloc.push((5, 1, 3));

        assert_eq!(alloc.get(&index), &(5, 1, 3));

        alloc.get_mut(&mut index).1 = 13;

        assert_eq!(alloc.get(&index), &(5, 13, 3));

        // This should still work
        alloc.pop_unsized(index);
    }
}
