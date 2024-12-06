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

use core::ptr;

use crate::println;
use crate::common::{is_aligned, KernelResult, Err};
use crate::memlayout::{ACLINT_SSWI_PADDR, CLINT, CLINT_SIZE, PLIC, PLIC_SIZE, UART0};
use crate::memory::{VirtAddr, PhysAddr, alloc_pages, PAGE_SIZE, __free_ram_end, __free_ram};
/* 64bit arch*/
pub const SATP_SV48: usize = 9 << 60;
pub const PAGE_V: usize = 1 << 0;
pub const PAGE_R: usize = 1 << 1;
pub const PAGE_W: usize = 1 << 2;
pub const PAGE_X: usize = 1 << 3;
pub const PAGE_U: usize = 1 << 4;

extern "C" {
    static __kernel_base: u8;
    static __text: u8;
    static __text_end: u8;
    static __data: u8;
    static __data_end: u8;
    static __rodata: u8;
    static __rodata_end: u8;
}

static mut KERNEL_VM: PageTableAddress = PageTableAddress::init();

fn to_k_vaddr(p_addr: PhysAddr) -> VirtAddr {
    let user_max: usize = 0x00007fffffffffff;
    let converter = !user_max;
    println!("{:x}", converter);
    let new_addr = converter | p_addr.addr;
    println!("{:x}", new_addr);
    VirtAddr::new(new_addr)
}

impl VirtAddr {
    #[inline]
    pub fn get_vpn(&self, idx: usize) -> usize {
        self.addr >> (12 + idx * 9) & 0x1ff
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageTableEntryAddress(usize);

impl PageTableEntryAddress {
    #[inline]
    pub unsafe fn is_valid(&self) -> bool {
        *(self.0 as *const usize) & PAGE_V != 0
    }

    #[inline]
    pub unsafe fn write(&mut self, content: usize) {
        *(self.0 as *mut usize) = content;
    }

    #[inline]
    pub unsafe fn get_pt(&self) -> PageTableAddress {
        PageTableAddress((*(self.0 as *const usize) << 2) & !0xfff)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageTableAddress(usize);

impl PageTableAddress {
    pub const fn init() -> Self {
        Self(0)
    }
    #[inline]
    pub unsafe fn get_pte(&self, vpn: usize) -> PageTableEntryAddress {
        let addr = (self.0 as *const usize).add(vpn);
        PageTableEntryAddress(addr as usize)
    }

    #[inline]
    pub fn get_address(&self) -> usize {
        self.0
    }
}

pub unsafe fn walk(page_table: PageTableAddress, vaddr: VirtAddr, alloc: bool) -> KernelResult<PageTableEntryAddress>  {
    let level = 3;
    _walk(page_table, vaddr, alloc, level)
}

unsafe fn _walk(page_table: PageTableAddress, vaddr: VirtAddr, alloc: bool, level: usize) -> KernelResult<PageTableEntryAddress> {
    let vpn = vaddr.get_vpn(level);
    let mut pte = page_table.get_pte(vpn);
    if vaddr.addr == 0xffff800001000000 {
        println!("{:?}, {:?}, {:?}, {:?}, {:?}", level, pte, page_table, vpn, pte.is_valid());
    }
    if level == 0 {
        Ok(pte)
    } else {
        if !pte.is_valid() {
            if !alloc {
                return Err(Err::PteNotFound)
            } else {
                let paddr = alloc_pages(1)?;
                pte.write(((paddr.addr / PAGE_SIZE) << 10) | PAGE_V);
            }
        };
        _walk(pte.get_pt(), vaddr, alloc, level - 1)
    }
}

pub unsafe fn map_page(root_table: PageTableAddress, vaddr: VirtAddr, paddr: PhysAddr, flags: usize) -> KernelResult<()> {
    assert!(is_aligned(vaddr.addr, PAGE_SIZE), "{:?}", vaddr);
    assert!(is_aligned(paddr.addr, PAGE_SIZE));
    let mut pte = walk(root_table, vaddr, true)?;
    if pte.is_valid() {
        println!("wow");
    }
    pte.write(((paddr.addr >> 12) << 10) | flags | PAGE_V);
    Ok(())

}

pub unsafe fn map_pages(root_table: PageTableAddress, vaddr: VirtAddr, paddr: PhysAddr, size: usize, flags: usize) -> KernelResult<()> {
    let mut offset = 0;
    while offset < size {
        map_page(root_table, vaddr + offset.into(), paddr + offset.into(), flags)?;
        offset += PAGE_SIZE
    };
    Ok(())
}

pub unsafe fn allocate_page_table() -> KernelResult<PageTableAddress> {
    let pt = PageTableAddress(alloc_pages(1)?.addr);
    ptr::copy(KERNEL_VM.0 as *const u8, pt.0 as *mut u8, PAGE_SIZE); 
    Ok(pt)
}

// function must be called only once.
pub unsafe fn kernel_vm_init() -> KernelResult<()> {
    let kernel_pt = PageTableAddress(alloc_pages(1)?.addr);
    let kerenel_txt = ptr::addr_of!(__text) as usize;
    let kerenel_txt_end = ptr::addr_of!(__text_end) as usize;
    let ro_datat = ptr::addr_of!(__rodata) as usize;
    let ro_datat_end = ptr::addr_of!(__rodata_end) as usize;
    let kernel_data = ptr::addr_of!(__data) as usize;
    let kernel_data_end = ptr::addr_of!(__data_end) as usize;
    let free_ram = ptr::addr_of!(__free_ram) as usize;
    let free_ram_end = ptr::addr_of!(__free_ram_end) as usize;

    to_k_vaddr(kerenel_txt.into());
    map_pages(kernel_pt, kerenel_txt.into(), kerenel_txt.into(), kerenel_txt_end - kerenel_txt, PAGE_R | PAGE_X)?;
    map_pages(kernel_pt, ro_datat.into(), ro_datat.into(), ro_datat_end - ro_datat, PAGE_R)?;
    map_pages(kernel_pt, kernel_data.into(), kernel_data.into(), kernel_data_end - kernel_data, PAGE_R | PAGE_W)?;
    map_pages(kernel_pt, free_ram.into(), free_ram.into(), free_ram_end - free_ram, PAGE_R | PAGE_W)?;

    map_pages(kernel_pt, UART0.into(), UART0.into(), PAGE_SIZE, PAGE_R | PAGE_W)?;
    // map_pages(kernel_pt, VIRTIO0.into(), VIRTIO0.into(), PAGE_SIZE, PAGE_R | PAGE_W)
    map_pages(kernel_pt, PLIC.into(), PLIC.into(), PLIC_SIZE, PAGE_R + PAGE_W)?;
    map_pages(kernel_pt, CLINT.into(), CLINT.into(), CLINT_SIZE, PAGE_R | PAGE_W)?;
    map_pages(kernel_pt, ACLINT_SSWI_PADDR.into(), ACLINT_SSWI_PADDR.into(), PAGE_SIZE, PAGE_R | PAGE_W)?;

    KERNEL_VM = kernel_pt;
    Ok(())
}
