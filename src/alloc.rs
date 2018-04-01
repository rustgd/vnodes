use std::cell::RefCell;
use std::marker::PhantomData;
use std::mem::{forget, size_of};
use std::ptr;

use util::{drop_maybe_sized, into_maybe_fat, into_maybe_fat_mut};

const ALIGN: usize = 0x1 << ALIGN_LOG;
const ALIGN_LOG: usize = 4;
const CAP: usize = 0x1 << 14;
const MAX_BYTES_PER_ALLOC: usize = MAX_CELLS_PER_ALLOC << ALIGN_LOG;
const MAX_CELLS_PER_ALLOC: usize = 0x1 << 8;
const NUM_CELLS: usize = CAP << ALIGN_LOG;

thread_local!(static ALLOC: RefCell<Allocator> = RefCell::new(Allocator::new()));

struct Allocator {
    pub raw: RawAllocator,
}

impl Allocator {
    pub fn new() -> Self {
        Allocator {
            raw: RawAllocator::new(),
        }
    }

    pub fn get<'a, T>(&self, index: &'a AllocIndex<T>) -> &'a T
    where
        T: Data + ?Sized,
    {
        unsafe {
            &*into_maybe_fat(
                self.raw.ptr(index.index as usize),
                T::bytes_to_len(index.size as usize),
            )
        }
    }

    pub fn get_mut<'a, T>(&mut self, index: &'a mut AllocIndex<T>) -> &'a mut T
    where
        T: Data + ?Sized,
    {
        unsafe {
            &mut *into_maybe_fat_mut(
                self.raw.ptr_mut(index.index as usize),
                T::bytes_to_len(index.size as usize),
            )
        }
    }

    pub fn push<T: Data + Sized>(&mut self, value: T) -> AllocIndex<T> {
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

    /// count = number of `T`s
    pub unsafe fn push_unsized<T, U: Data + ?Sized>(
        &mut self,
        data: *const T,
        count: usize,
    ) -> AllocIndex<U> {
        let size = count * size_of::<T>();
        let index = self.raw.allocate(size);

        ptr::copy_nonoverlapping(data, self.raw.ptr_mut(index) as *mut T, count);

        let index = index as u32;
        let size = size as u32;

        AllocIndex {
            index,
            size,
            marker: PhantomData,
        }
    }

    pub fn pop<T: Data + Sized>(&mut self, index: AllocIndex<T>) -> T {
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

    pub fn pop_unsized<T: Data + ?Sized>(&mut self, index: AllocIndex<T>) {
        unsafe {
            let ptr = self.raw.ptr_mut(index.index as usize);
            let len = T::bytes_to_len(index.size as usize);
            drop_maybe_sized::<T>(ptr, len);
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
pub struct AllocIndex<T: Data + ?Sized> {
    index: u32,
    size: u32,
    marker: PhantomData<*const T>, // not thread-safe
}

impl<T: Data + ?Sized> Drop for AllocIndex<T> {
    fn drop(&mut self) {
        pop_unsized(AllocIndex {
            index: self.index,
            size: self.size,
            marker: self.marker,
        });
    }
}

pub unsafe trait Data {
    fn bytes_to_len(bytes: usize) -> usize;
}

unsafe impl<T: Sized> Data for T {
    fn bytes_to_len(bytes: usize) -> usize {
        bytes
    }
}

unsafe impl<T> Data for [T] {
    fn bytes_to_len(bytes: usize) -> usize {
        bytes / size_of::<T>()
    }
}

struct RawAllocator {
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
        assert!(bytes > 0);

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

pub fn get<'a, T>(index: &'a AllocIndex<T>) -> &'a T
where
    T: Data + ?Sized,
{
    ALLOC.with(move |alloc| alloc.borrow().get(index))
}

pub fn get_mut<'a, T>(index: &'a mut AllocIndex<T>) -> &'a mut T
where
    T: Data + ?Sized,
{
    ALLOC.with(move |alloc| alloc.borrow_mut().get_mut(index))
}

pub fn push<T>(value: T) -> AllocIndex<T> {
    ALLOC.with(|alloc| alloc.borrow_mut().push(value))
}

pub fn pop<T>(index: AllocIndex<T>) -> T {
    ALLOC.with(|alloc| alloc.borrow_mut().pop(index))
}

pub fn pop_unsized<T: Data + ?Sized>(index: AllocIndex<T>) {
    ALLOC.with(|alloc| alloc.borrow_mut().pop_unsized(index));
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

    #[test]
    fn test_coerce_unsized() {
        let mut alloc = Allocator::new();
        let index: AllocIndex<[i32; 10]> = alloc.push([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        //let mut index = index as AllocIndex<[i32]>;

        // TODO: this does not work with current stable Rust
        // TODO: a conversion from `AllocIndex<Sized>` to `AllocIndex<?Sized>` needs to be made
        // TODO: but `CoerceUnsized` is unstable

        alloc.pop_unsized(index);
    }

    #[test]
    fn test_boxed_unsized() {
        let mut alloc = Allocator::new();
        let boxed = Box::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]) as Box<[i32]>;
        let index: AllocIndex<[i32]> = alloc.push_boxed(boxed);

        assert_eq!(alloc.get(&index).iter().sum::<i32>(), 55);

        alloc.pop_unsized(index);
    }
}
