mod paging;

use core::alloc::GlobalAlloc;
use core::cell::RefCell;
use core::sync::atomic;
use core::sync::atomic::AtomicBool;
use core::ptr;

type FlagElemType = u64;

const MEMORY_BLOCK_SIZE: usize = 4096;
const FLAG_BITS: usize = FlagElemType::BITS as usize;
const STATIC_HEAP_SIZE: usize = 512 * 1024 * 1024; // 512MiB
const HEAP_BITMAP_SIZE: usize = STATIC_HEAP_SIZE / (MEMORY_BLOCK_SIZE * FLAG_BITS);

// GLOBAL_ALLOCATORが使用中かどうかを示す
// trueなら使用中
static ALLOCATOR_SEMAPHORE: AtomicBool = AtomicBool::new(false);

#[global_allocator]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator { 
    head: [0; STATIC_HEAP_SIZE], 
    bitmap: RefCell::new([0; HEAP_BITMAP_SIZE]),
}; 

#[alloc_error_handler]
fn alloc_error(_: core::alloc::Layout) -> ! {
  loop {}
}

#[repr(align(4096))]
struct GlobalAllocator {
  head: [u8; STATIC_HEAP_SIZE],
  bitmap: RefCell<[FlagElemType; HEAP_BITMAP_SIZE]>
}

unsafe impl Sync for GlobalAllocator {}

unsafe impl GlobalAlloc for GlobalAllocator {
  unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
    let size = layout.size();
    let entries = (size + MEMORY_BLOCK_SIZE - 1) / MEMORY_BLOCK_SIZE;
    self.semaphore_ops(|allocator| {
      let mut alloc_from = 0;
      let mut allocatable = 0;

      let mut bitmap = allocator.bitmap.borrow_mut();
      // 得られたentriesを連続して確保できる領域が存在するか探索する
      'alloc_search: for (idx, &flg) in bitmap.iter().enumerate() {
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

          bitmap[blk_idx] |= 1_u64 << bit_idx;
        }

        (allocator.head.as_ptr() as usize + alloc_from * MEMORY_BLOCK_SIZE) as *mut u8
      };

      allocated_ptr
    })
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
    let head_offset = ptr as usize - self.head.as_ptr() as usize;
    let head_idx = head_offset / MEMORY_BLOCK_SIZE;
    let entries = (layout.size() + MEMORY_BLOCK_SIZE - 1) / MEMORY_BLOCK_SIZE;

    core::ptr::write_bytes(ptr, 0, entries * MEMORY_BLOCK_SIZE);

    self.semaphore_ops(|allocator| {
      let mut bitmap = allocator.bitmap.borrow_mut();

      for offset in 0..entries {
        let blk_idx = (head_idx + offset) / FLAG_BITS;
        let bit_idx = (head_idx + offset) & FLAG_BITS;

        bitmap[blk_idx] &= !(1 << bit_idx);
      }
    });
  }
}

impl GlobalAllocator {
  fn semaphore_ops<T>(&self, f: impl Fn(&Self) -> T) -> T{
    while let Err(_) = ALLOCATOR_SEMAPHORE.compare_exchange(
      false, 
      true, 
      atomic::Ordering::AcqRel,
      atomic::Ordering::Acquire
    ) {}

    let ret = f(self);

    ALLOCATOR_SEMAPHORE.store(false, atomic::Ordering::Release);

    ret
  }
}