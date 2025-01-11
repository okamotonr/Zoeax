use crate::address::{PhysAddr, PAGE_SIZE};
use core::ptr;
/// Physical Page Allocator
/// Allocated Page will never be reclaimed
pub struct BumpAllocator {
    start_addr: PhysAddr,
    end_addr: PhysAddr,
}

impl BumpAllocator {
    pub unsafe fn new(free_ram_phys: usize, free_ram_end_phys: usize) -> Self {
        assert!(free_ram_phys < free_ram_end_phys);

        ptr::write_bytes(
            free_ram_phys as *mut u8,
            0,
            free_ram_end_phys - free_ram_phys,
        );
        Self {
            start_addr: free_ram_phys.into(),
            end_addr: free_ram_end_phys.into(),
        }
    }

    pub fn allocate_page(&mut self) -> PhysAddr {
        self.allocate_pages(1)
    }

    pub fn allocate_pages(&mut self, page_num: usize) -> PhysAddr {
        let ret = self.start_addr;
        self.start_addr = self.start_addr.add(PAGE_SIZE * page_num);
        assert!(self.start_addr <= self.end_addr);
        ret
    }

    pub fn end_allocation(self) -> (PhysAddr, PhysAddr) {
        (self.start_addr, self.end_addr)
    }
}
