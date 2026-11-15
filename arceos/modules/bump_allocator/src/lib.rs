#![no_std]

use allocator::{AllocError, AllocResult, BaseAllocator, ByteAllocator, PageAllocator};
use core::alloc::Layout;
use core::ptr::NonNull;

#[inline]
fn align_up(addr: usize, align: usize) -> Option<usize> {
    addr.checked_add(align - 1).map(|v| v & !(align - 1))
}

#[inline]
fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const SIZE: usize> {
    start: usize,
    end: usize,
    b_pos: usize,
    p_pos: usize,
    alloc_count: usize,
}

impl<const SIZE: usize> EarlyAllocator<SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            b_pos: 0,
            p_pos: 0,
            alloc_count: 0,
        }
    }
}

impl<const SIZE: usize> BaseAllocator for EarlyAllocator<SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.b_pos = start;
        self.p_pos = start + size;
        self.alloc_count = 0;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        if start >= self.end {
            // Extend the range forwards. The page pointer follows the new end.
            self.end = start + size;
            self.p_pos = self.end;
            Ok(())
        } else if start + size <= self.start {
            // Extend the range backwards. The byte pointer follows the new start.
            self.start = start;
            self.b_pos = start;
            Ok(())
        } else {
            Err(AllocError::MemoryOverlap)
        }
    }
}

impl<const SIZE: usize> ByteAllocator for EarlyAllocator<SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let alloc_start = align_up(self.b_pos, layout.align()).ok_or(AllocError::NoMemory)?;
        let alloc_end = alloc_start
            .checked_add(layout.size())
            .ok_or(AllocError::NoMemory)?;

        if alloc_end > self.p_pos {
            return Err(AllocError::NoMemory);
        }

        self.b_pos = alloc_end;
        self.alloc_count += 1;
        NonNull::new(alloc_start as *mut u8).ok_or(AllocError::NotAllocated)
    }

    fn dealloc(&mut self, _pos: NonNull<u8>, _layout: Layout) {
        if self.alloc_count == 0 {
            return;
        }
        self.alloc_count -= 1;
        if self.alloc_count == 0 {
            // All byte allocations are gone: reclaim the whole byte area.
            self.b_pos = self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.b_pos - self.start
    }

    fn available_bytes(&self) -> usize {
        self.p_pos - self.b_pos
    }
}

impl<const SIZE: usize> PageAllocator for EarlyAllocator<SIZE> {
    const PAGE_SIZE: usize = SIZE;

    fn alloc_pages(
        &mut self,
        num_pages: usize,
        align_pow2: usize,
    ) -> AllocResult<usize> {
        let align = if align_pow2 == 0 { 1 } else { align_pow2 };
        if !align.is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }

        let need = num_pages.checked_mul(SIZE).ok_or(AllocError::NoMemory)?;
        if need == 0 || need > self.p_pos.saturating_sub(self.b_pos) {
            return Err(AllocError::NoMemory);
        }

        let new_p_pos = align_down(self.p_pos - need, align);
        if new_p_pos < self.b_pos {
            return Err(AllocError::NoMemory);
        }

        self.p_pos = new_p_pos;
        Ok(new_p_pos)
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        // For the pages area, it will never be freed!
    }

    fn total_pages(&self) -> usize {
        (self.end - self.start) / SIZE
    }

    fn used_pages(&self) -> usize {
        (self.end - self.p_pos) / SIZE
    }

    fn available_pages(&self) -> usize {
        (self.p_pos - self.b_pos) / SIZE
    }
}
