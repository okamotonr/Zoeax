/*
 Implement sv48;

virtual address
 47        39  38       30  29       21  20       12  11               0
|   VPN[3]   |   VPN[2]   |   VPN[1]   |   VPN[0]   |    page offset    |
     9            9            9            9                12


physical address
 55                39  38       30  29       21  20       12  11               0
|       PPN[3]       |   PPN[2]   |   PPN[1]   |   PPN[0]   |    page offset    |
        17                9            9            9                12


page table entry
 53                37  36       28  27       19  18       10
|       PPN[3]       |   PPN[2]   |   PPN[1]   |   PPN[0]   |
        17                9            9            9
9    8  7 6 5 4 3 2 1 0
| RSW |D|A|G|U|X|W|R|V|
 *
 *
 */

use crate::{
    address::{KernelVAddress, PhysAddr, VirtAddr, PAGE_SIZE},
    common::{ErrKind, KernelResult},
    kerr,
    memlayout::KERNEL_CODE_PFX,
};

use core::ptr;
use core::{
    arch::asm,
    ops::{Deref, DerefMut},
};

pub const SATP_SV48: usize = 9 << 60;
pub const PAGE_V: usize = 1 << 0;
pub const PAGE_R: usize = 1 << 1;
pub const PAGE_W: usize = 1 << 2;
pub const PAGE_X: usize = 1 << 3;
pub const PAGE_U: usize = 1 << 4;

// TODO: use once cell
pub static mut KERNEL_VM_ROOT: PageTable = PageTable::new();
pub static mut LV2TABLE: PageTable = PageTable::new();

// page table lv1(bottom) has 512 * 4kb page = 2048kb
// page table lv2(middle) has 512 * lv1 table = 512 * 2048kb
// ...

// TODO: root page table and other tables should be different type?
#[repr(align(4096))]
pub struct PageTable([PTE; 512]);

impl PageTable {
    pub const fn new() -> Self {
        Self([PTE(0); 512])
    }
    pub fn map(&self, parent: &mut Self, vaddr: VirtAddr) -> KernelResult<usize> {
        let (level, entry) = parent.walk(vaddr);
        if level == 0 {
            Err(kerr!(ErrKind::VaddressAlreadyMapped))
        } else {
            entry.write(KernelVAddress::from(self as *const _), PAGE_V);
            Ok(level - 1)
        }
    }

    pub fn walk(&mut self, vaddr: VirtAddr) -> (usize, &mut PTE) {
        let mut page_table = self;
        // walk page table
        for level in (1..=3).rev() {
            let vpn = vaddr.get_vpn(level);
            let pte = &mut page_table[vpn];
            if !pte.is_valid() {
                return (level, pte);
            }
            page_table = pte.as_page_table();
        }

        let pte = &mut page_table[vaddr.get_vpn(0)];
        (0, pte)
    }

    pub unsafe fn activate(&self) {
        let addr: PhysAddr = KernelVAddress::from(self as *const Self).into();
        asm!(
            "sfence.vma x0, x0",
            "csrw satp, {satp}",
            "sfence.vma x0, x0",
            satp = in(reg) (SATP_SV48 | (addr.addr >> 12))
        )
    }

    pub fn copy_global_mapping(&mut self) {
        let self_addr = self as *mut PageTable as *mut u8;
        unsafe {
            let k_root = &raw const KERNEL_VM_ROOT as *const u8;
            ptr::copy::<u8>(k_root, self_addr, PAGE_SIZE);
        };
    }

    pub unsafe fn activate_kernel_table() {
        let address = (&raw const KERNEL_VM_ROOT as *const _ as usize) & !KERNEL_CODE_PFX;
        unsafe {
            asm!(
                "sfence.vma x0, x0",
                "csrw satp, {satp}",
                "sfence.vma x0, x0",
                satp = in(reg) (SATP_SV48 | (address >> 12))
            )
        }
    }
}

impl Deref for PageTable {
    type Target = [PTE; 512];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PageTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// 4kb page
#[repr(align(4096))]
#[derive(Debug, Clone, Copy, Default)]
pub struct Page;

impl Page {
    pub fn new() -> Self {
        Self
    }
    pub fn map(&self, parent: &mut PageTable, vaddr: VirtAddr, flags: usize) -> KernelResult<()> {
        let (level, entry) = parent.walk(vaddr);
        if level != 0 {
            Err(kerr!(ErrKind::PageTableNotMappedYet))
        } else if entry.is_valid() {
            Err(kerr!(ErrKind::VaddressAlreadyMapped))
        } else {
            entry.write(KernelVAddress::from(self as *const _), flags | PAGE_V);
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PTE(usize);

impl PTE {
    pub fn is_valid(&self) -> bool {
        self.0 & PAGE_V != 0
    }

    pub fn write<A: Into<PhysAddr>>(&mut self, addr: A, flags: usize) {
        let phys: PhysAddr = addr.into();
        let addr = phys.addr;
        self.0 = ((addr >> 12) << 10) | flags;
    }

    pub fn as_page_table(&mut self) -> &mut PageTable {
        let phys_addr = PhysAddr::from((self.0 << 2) & !0xfff);
        let raw: *mut PageTable = KernelVAddress::from(phys_addr).into();
        unsafe { &mut *raw }
    }
}
