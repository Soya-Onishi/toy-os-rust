mod paging;

use core::alloc::GlobalAlloc;
use core::cell::UnsafeCell;
use core::ptr;
use core::arch::asm;
use core::matches;

use bootloader_api::info;

type FlagElemType = u64;

const MEMORY_BLOCK_SIZE: usize = 4096;
const FLAG_BITS: usize = FlagElemType::BITS as usize;
const INIT_ALLOC_SIZE: usize = 16 * 1024 * 1024;
const INIT_BITMAP_SIZE: usize = INIT_ALLOC_SIZE / (MEMORY_BLOCK_SIZE * FLAG_BITS);

static mut init_alloc_bitmap: [FlagElemType; INIT_BITMAP_SIZE] = [0; INIT_BITMAP_SIZE];

#[global_allocator]
static mut GLOBAL_ALLOCATOR: BitMapAllocator = BitMapAllocator { 
    head: 0, 
    capacity: 0, 
    used_flags: core::cell::UnsafeCell::new(core::ptr::null_mut())
}; 

pub struct BitMapAllocator {
  head: usize,
  capacity: usize,
  used_flags: UnsafeCell<*mut FlagElemType>,
}

unsafe impl Sync for BitMapAllocator {}

unsafe impl GlobalAlloc for BitMapAllocator {
  unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
    let size = layout.size();
    let entries = (size + MEMORY_BLOCK_SIZE - 1) / MEMORY_BLOCK_SIZE;
    let mut alloc_from = 0;
    let mut allocatable = 0;

    asm!("cli");
    let used_flags = core::slice::from_raw_parts_mut(*self.used_flags.get(), self.capacity / MEMORY_BLOCK_SIZE);

    // 得られたentriesを連続して確保できる領域が存在するか探索する
    'alloc_search: for (idx, flg) in used_flags.iter().enumerate() {
      for i in 0..FLAG_BITS {
        if (flg & (1 << i)) == 0 {
          if allocatable == 0 {
            alloc_from = idx * FLAG_BITS + i; 
          }

          allocatable += 1;
        } else {
          allocatable = 0;
        }

        if allocatable >= entries {
          break 'alloc_search;
        }
      }
    }

    let allocated_ptr = if allocatable < entries {
      ptr::null_mut()
    } else {
      // 実際に領域を確保する
      for offset in 0..entries {
        let blk_idx = (alloc_from + offset) / FLAG_BITS;
        let bit_idx = (alloc_from + offset) % FLAG_BITS;

        used_flags[blk_idx] |= 1_u64 << bit_idx;
      }

      (self.head + alloc_from * MEMORY_BLOCK_SIZE) as *mut u8
    };
    
    asm!("sti");

    allocated_ptr
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
    let head_idx = ptr as usize / MEMORY_BLOCK_SIZE;
    let entries = (layout.size() + MEMORY_BLOCK_SIZE - 1) / MEMORY_BLOCK_SIZE;

    asm!("cli");
    let used_flags = core::slice::from_raw_parts_mut(*self.used_flags.get(), self.capacity / MEMORY_BLOCK_SIZE);
    for offset in 0..entries {
      let blk_idx = (head_idx + offset) / FLAG_BITS;
      let bit_idx = (head_idx + offset) & FLAG_BITS;

      used_flags[blk_idx] &= !(1 << bit_idx);
    } 
    asm!("sti");
  }
}

impl BitMapAllocator {
  pub fn new(regions: &info::MemoryRegions) -> BitMapAllocator {
    let region = regions.iter()
      .filter(|&r| matches!(r.kind, info::MemoryRegionKind::Usable))
      .max_by(|&r0, &r1| {
        let r0_size = r0.start - r0.end;
        let r1_size = r1.start - r1.end;
        r0_size.cmp(&r1_size)
      })
      .unwrap();
        
    const USED_FLG_BLK_ALIGN: usize = FLAG_BITS * MEMORY_BLOCK_SIZE;

    let head = region.start as usize;
    let head_aligned = (head + MEMORY_BLOCK_SIZE - 1) / MEMORY_BLOCK_SIZE * MEMORY_BLOCK_SIZE;

    let len = region.end as usize - head_aligned;
    let len_aligned = len / MEMORY_BLOCK_SIZE * MEMORY_BLOCK_SIZE;
    if len_aligned < MEMORY_BLOCK_SIZE {
      panic!("max memory region is less than MEMORY_BLOCK_SIZE");
    }

    let require_for_used_flg = (len_aligned + USED_FLG_BLK_ALIGN - 1) / USED_FLG_BLK_ALIGN;
    let used_flags = unsafe { core::slice::from_raw_parts_mut(head_aligned as *mut u64, require_for_used_flg) };
    for idx in 0..require_for_used_flg {
      let blk_idx = idx / FLAG_BITS;
      let bit_idx = idx % FLAG_BITS;
      
      used_flags[blk_idx] |= 1 << bit_idx;
    }

    let unsable_tail_count = FLAG_BITS - ((len_aligned / MEMORY_BLOCK_SIZE) % FLAG_BITS);
    for idx in 0..unsable_tail_count {
      used_flags[require_for_used_flg - 1] |= 1 << (FLAG_BITS - 1 - idx);
    }

    let used_flags = UnsafeCell::new(used_flags.as_mut_ptr());

    BitMapAllocator { head: head_aligned, capacity: len_aligned, used_flags }
  }

  pub fn new_for_init(regions: &info::MemoryRegions) -> BitMapAllocator { 
    fn align_blocksize(addr: usize) -> usize { 
      (addr + MEMORY_BLOCK_SIZE - 1) / MEMORY_BLOCK_SIZE * MEMORY_BLOCK_SIZE
    }

    let region = regions.iter_mut()
      .filter(|r| matches!(r.kind, info::MemoryRegionKind::Usable))
      .find(|r| {
        let start = align_blocksize(r.start as usize)
        let end = r.end as usize;

        end - start >= INIT_ALLOC_SIZE
      })
      .expect("there is no space for initial allocation");

    let head = align_blocksize(region.start as usize);
    let capacity = region.end as usize - head;
    let used_flags = UnsafeCell::new(unsafe{ &mut init_alloc_bitmap as *mut FlagElemType});
    BitMapAllocator { head, capacity, used_flags }
  }
}

pub fn set_allocator(allocator: BitMapAllocator) {
  unsafe { GLOBAL_ALLOCATOR = allocator }
}