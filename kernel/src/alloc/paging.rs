const PAGE_ENTRY_NUM: usize = 512;

static mut init_level_3: PageTable = PageTable { entries: [PageEntry { entry: 0 }; PAGE_ENTRY_NUM] };
static mut init_level_2: PageTable = PageTable { entries: [PageEntry { entry: 0 }; PAGE_ENTRY_NUM] };
static mut init_level_1: PageTable = PageTable { entries: [PageEntry { entry: 0 }; PAGE_ENTRY_NUM] };

#[repr(C)]
struct  PageTable {
  entries: [PageEntry; PAGE_ENTRY_NUM],
}

#[derive(Clone, Copy)]
#[repr(transparent)]
struct PageEntry {
  entry: u64,
}

fn init_pagetable() {

}