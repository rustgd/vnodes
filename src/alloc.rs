use std::cell::UnsafeCell;

const ALIGN: usize = 0x1 << ALIGN_LOG;
const ALIGN_LOG: usize = 4;
const CAP: usize = 0x1 << 14;
const MAX_BYTES_PER_ALLOC: usize = MAX_CELLS_PER_ALLOC << ALIGN_LOG;
const MAX_CELLS_PER_ALLOC: usize = 0x1 << 8;
const NUM_CELLS: usize = CAP << ALIGN_LOG;

pub struct Allocator {
    buffer: Box<[UnsafeCell<[u8; ALIGN]>]>,
    prev_size: Vec<usize>,
    cursor: usize,
}

impl Allocator {
    pub fn new() -> Self {
        Allocator {
            buffer: (0..NUM_CELLS)
                .map(|_| UnsafeCell::new([0; ALIGN]))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            cursor: 0,
            prev_size: vec![0],
        }
    }

    pub fn allocate(&mut self, bytes: usize) -> *mut () {
        let cells = aligned_cells(bytes);
        let start = self.cursor;
        let end = start + cells;

        assert!(NUM_CELLS > end, "Allocator went out of memory");

        self.cursor += cells;

        ensure_index(&mut self.prev_size, self.cursor);
        self.prev_size[self.cursor] = cells;

        self.buffer[start].get() as *mut ()
    }

    pub unsafe fn deallocate(&mut self, ptr: *mut (), size: usize) {
        let index = ptr as usize - (self.buffer[0].get() as usize);
        let index = index >> ALIGN_LOG;
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

    pub unsafe fn debug(&mut self) {
        println!(
            "Allocator {{ buffer: {:#?}, prev_size: {:?}, cursor: {} }}",
            /*(*self.buffer.get())[0..self.cursor]
                .iter()
                .map(|cell| format!("[0x{:x}, 0x{:x}]", conv64(&cell[0..8]), conv64(&cell[8..16])))
                .collect::<Vec<_>>(),*/
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
