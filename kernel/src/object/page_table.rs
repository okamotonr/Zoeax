use crate::{common::{KernelResult, Err}, memory::VirtAddr};

pub struct PageTable;

pub const SATP_SV48: usize = 9 << 60;
pub const PAGE_V: usize = 1 << 0;
pub const PAGE_R: usize = 1 << 1;
pub const PAGE_W: usize = 1 << 2;
pub const PAGE_X: usize = 1 << 3;
pub const PAGE_U: usize = 1 << 4;


// TODO: use array [PTE; 512]
impl PageTable {
    pub fn map(&self, parent: &mut Self, vaddr: VirtAddr) -> KernelResult<()> {
        let (level, entry) = parent.walk(vaddr);
        if level == 0 {
            Err(Err::VaddressAlreadyMapped)
        } else {
            entry.write(self as *const _ as usize, PAGE_V);
            Ok(())
        }
    }

    fn get_pte(&mut self, vpn: usize) -> &mut PTE {
        unsafe {
            (self as *mut Self).cast::<PTE>().add(vpn).as_mut().unwrap()
        }
    }

    fn walk(&mut self, vaddr: VirtAddr) -> (usize, &mut PTE) {
        let mut page_table = self;
        // walk page table 
        for level in (1..=3).rev() {
            let vpn = vaddr.get_vpn(level);
            let pte = page_table.get_pte(vpn);
            if !pte.is_valid() {
                return (level, pte)
            }
            page_table = pte.as_page_table();
        }

        let pte = page_table.get_pte(vaddr.get_vpn(0));
        (0, pte)
    }
}

// 4kb page
pub struct Page;

impl Page {
    pub fn map(&self, parent: &mut PageTable, vaddr: VirtAddr, flags: usize) -> KernelResult<()> {
        let (level, entry) = parent.walk(vaddr);
        if level != 0 {
            Err(Err::PageTableNotMappedYet)
        } else {
            if entry.is_valid() {
                Err(Err::VaddressAlreadyMapped)
            } else {
                entry.write(self as *const _ as usize, flags | PAGE_V);
                Ok(())
            }
        }
    }
}

pub struct PTE(usize);

impl PTE {
    pub fn is_valid(&self) -> bool {
        self.0 & PAGE_V != 0
    }

    pub fn write(&mut self, addr: usize, flags: usize) {
        self.0 = ((addr >> 12)  << 10) | flags
    }

    pub fn as_page_table(&mut self) -> &mut PageTable {
        let raw = (self.0 << 2) & !0xfff;
        unsafe {
            (raw as *mut PageTable).as_mut().unwrap()
        }
    }
}

