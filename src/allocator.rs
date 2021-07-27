#![allow(clippy::missing_inline_in_public_items)]
use crate::{spinlock::SpinLock, syscalls};
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr,
};

// Allocates in chunks of 64 bytes. The `usage_mask` is a bitmask that is 1 where something is
// allocated.
pub struct SmallAllocator {
    slab: Slab,
    usage_mask: u64,
}

#[repr(align(64))]
struct Slab([u8; 4096]);

impl core::ops::Deref for Slab {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl core::ops::DerefMut for Slab {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.0[..]
    }
}

impl SmallAllocator {
    fn to_blocks(size: usize) -> usize {
        let remainder = size % 64;
        let size = if remainder == 0 {
            size
        } else {
            size + 64 - remainder
        };
        size / 64
    }

    pub fn alloc(&mut self, layout: Layout) -> *mut u8 {
        if layout.align() > 64 || layout.size() == 0 || layout.size() > 4096 {
            return core::ptr::null_mut();
        }

        let blocks = Self::to_blocks(layout.size());

        let my_mask = u64::MAX << (64 - blocks);

        for i in 0..=(64 - blocks) {
            if ((my_mask >> i) & self.usage_mask) == 0 {
                self.usage_mask |= my_mask >> i;
                return self.slab[64 * i..].as_mut_ptr();
            }
        }

        core::ptr::null_mut()
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) -> bool {
        let offset = ptr.offset_from(self.slab.as_mut_ptr());
        if !(0..4096).contains(&offset) {
            return false;
        }

        let offset_blocks = offset as usize / 64;
        let blocks = Self::to_blocks(layout.size());

        let my_mask = (u64::MAX << (64 - blocks)) >> offset_blocks;

        assert!(my_mask & self.usage_mask == my_mask);
        self.usage_mask ^= my_mask;

        true
    }
}

pub struct Allocator {
    cache: SpinLock<[(bool, *mut u8, usize); 64]>,
    small: SpinLock<SmallAllocator>,
}

impl Allocator {
    pub const fn new() -> Self {
        Self {
            cache: SpinLock::new([(false, ptr::null_mut(), 0); 64]),
            small: SpinLock::new(SmallAllocator {
                slab: Slab([0u8; 4096]),
                usage_mask: 0,
            }),
        }
    }
}

fn round_to_page(layout: Layout) -> Layout {
    let remainder = layout.size() % 4096;
    let size = if remainder == 0 {
        layout.size()
    } else {
        layout.size() + 4096 - remainder
    };
    match Layout::from_size_align(size, layout.align()) {
        Ok(l) => l,
        Err(_) => alloc::alloc::handle_alloc_error(layout),
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, mut layout: Layout) -> *mut u8 {
        let small_ptr = self.small.lock().alloc(layout);
        if !small_ptr.is_null() {
            return small_ptr;
        }

        layout = round_to_page(layout);

        let mut cache = self.cache.lock();

        // Find the closest fit
        if let Some((is_used, ptr, _)) = cache
            .iter_mut()
            .filter(|(is_used, _, len)| !*is_used && *len >= layout.size())
            .min_by_key(|(_, _, len)| *len - layout.size())
        {
            *is_used = true;
            return *ptr;
        }

        // We didn't find a mapping that's already big enough, resize the largest one.
        if let Some((is_used, ptr, len)) = cache
            .iter_mut()
            .filter(|(is_used, ptr, _)| !*is_used && !ptr.is_null())
            .max_by_key(|(_, _, len)| *len)
        {
            *is_used = true;
            *ptr = syscalls::mremap(*ptr, *len, layout.size(), libc::MREMAP_MAYMOVE)
                .unwrap_or(core::ptr::null_mut());
            return *ptr;
        }

        syscalls::mmap(
            core::ptr::null_mut(),
            layout.size(),
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_ANON | libc::MAP_PRIVATE,
            -1,
            0,
        )
        .unwrap_or(core::ptr::null_mut())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, mut layout: Layout) {
        if self.small.lock().dealloc(ptr, layout) {
            return;
        }

        layout = round_to_page(layout);

        let mut cache = self.cache.lock();

        // Look for this entry in the cache and mark it as unused
        for (is_used, cache_ptr, _len) in cache.iter_mut() {
            if ptr == *cache_ptr {
                *is_used = false;
                return;
            }
        }

        // We didn't find it in the cache, try to add it
        for (is_used, cache_ptr, len) in cache.iter_mut() {
            if !*is_used {
                *cache_ptr = ptr;
                *len = layout.size();
                return;
            }
        }
        // This is technically fallible, but it seems like there isn't a way to indicate failure
        // when deallocating.
        let _ = syscalls::munmap(ptr, layout.size());
    }

    unsafe fn realloc(&self, ptr: *mut u8, mut layout: Layout, mut new_size: usize) -> *mut u8 {
        let mut small = self.small.lock();
        let offset = ptr.offset_from(small.slab.as_mut_ptr());
        if (0..4096).contains(&offset) {
            drop(small);
            let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
            let new_ptr = self.alloc(new_layout);
            core::ptr::copy_nonoverlapping(ptr, new_ptr, core::cmp::min(layout.size(), new_size));
            self.dealloc(ptr, layout);
            return new_ptr;
        }

        layout = round_to_page(layout);
        let remainder = new_size % 4096;
        new_size = if remainder == 0 {
            new_size
        } else {
            new_size + 4096 - remainder
        };

        if layout.size() >= new_size {
            return ptr;
        }

        let mut cache = self.cache.lock();

        for (is_used, cache_ptr, len) in cache.iter_mut() {
            if *cache_ptr == ptr {
                *len = new_size;
                assert!(*is_used);
                *cache_ptr = syscalls::mremap(ptr, layout.size(), new_size, libc::MREMAP_MAYMOVE)
                    .unwrap_or(core::ptr::null_mut());
                return *cache_ptr;
            }
        }

        syscalls::mremap(ptr, layout.size(), new_size, libc::MREMAP_MAYMOVE)
            .unwrap_or(core::ptr::null_mut())
    }
}

impl Drop for Allocator {
    fn drop(&mut self) {
        for (_, ptr, len) in self.cache.lock().iter() {
            unsafe {
                let _ = syscalls::munmap(*ptr, *len);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small() {
        let mut alloc = SmallAllocator {
            slab: Slab([0u8; 4096]),
            usage_mask: 0,
        };

        for size in 1..4096 {
            let remainder = size % 64;
            let rounded = if remainder == 0 {
                size
            } else {
                size + 64 - remainder
            };
            let blocks = rounded / 64;

            assert!(blocks * 64 >= size);

            let layout = Layout::from_size_align(size, 1).unwrap();

            let ptr = alloc.alloc(layout);

            assert!(!ptr.is_null());

            assert_eq!(blocks, alloc.usage_mask.count_ones() as usize);

            unsafe {
                assert!(alloc.dealloc(ptr, layout));
            }

            assert_eq!(0, alloc.usage_mask.count_ones());
        }
    }
}
