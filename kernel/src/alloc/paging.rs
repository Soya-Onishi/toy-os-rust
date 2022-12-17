const PAGE_ENTRY_NUM: usize = 512;

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