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

use crate::common::{is_aligned, Err, KernelResult};
// use crate::memlayout::{ACLINT_SSWI_PADDR, CLINT, CLINT_SIZE, PLIC, PLIC_SIZE, UART0};
use crate::memory::{
    alloc_pages, Address, PhysAddr, VirtAddr, PAGE_SIZE,
};
use crate::println;
use core::arch::asm;
/* 64bit arch*/
pub const SATP_SV48: usize = 9 << 60;
pub const PAGE_V: usize = 1 << 0;
pub const PAGE_R: usize = 1 << 1;
pub const PAGE_W: usize = 1 << 2;
pub const PAGE_X: usize = 1 << 3;
pub const PAGE_U: usize = 1 << 4;

// TODO: get from linker script or
pub const KERNEL_CODE_PFX: usize = 0xffffffff00000000;

//
pub const KERNEL_V_ADDR_PFX: usize = 0xffff800000000000;

extern "C" {
    static __text: u8;
    static __text_end: u8;
    static __data: u8;
    static __data_end: u8;
    static __rodata: u8;
    static __rodata_end: u8;
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct KVirtual;

pub type KernelVAddress = Address<KVirtual>;

impl From<PhysAddr> for KernelVAddress {
    fn from(value: PhysAddr) -> Self {
        Self::new(value.addr | KERNEL_V_ADDR_PFX)
    }
}

impl Into<PhysAddr> for KernelVAddress {
    fn into(self) -> PhysAddr {
        PhysAddr::new(self.addr & !KERNEL_V_ADDR_PFX)
    }
}

impl Into<VirtAddr> for KernelVAddress {
    fn into(self) -> VirtAddr {
        VirtAddr::new(self.addr)
    }
}

static mut KERNEL_VM: PageTable = PageTable(PhysAddr::new(0));

pub fn alloc_vm() -> KernelResult<KernelVAddress> {
    let paddr = alloc_pages(1)?;
    Ok(paddr.into())
}

impl VirtAddr {
    #[inline]
    pub fn get_vpn(&self, idx: usize) -> usize {
        self.addr >> (12 + idx * 9) & 0x1ff
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageTableEntry(PhysAddr);

impl PageTableEntry {
    #[inline]
    pub unsafe fn is_valid(&self, use_paddr: bool) -> bool {
        let ptr: *mut usize = if use_paddr {
            self.0.into() } else {
                let vaddr: KernelVAddress = self.0.into();
                vaddr.into()
        };
        *ptr & PAGE_V != 0
    }

    #[inline]
    pub unsafe fn write(&mut self, content: usize, use_paddr: bool) {
        let ptr: *mut usize = if use_paddr {
            self.0.into() } else {
                let vaddr: KernelVAddress = self.0.into();
                vaddr.into()
        };
        *ptr = content;
    }

    #[inline]
    pub unsafe fn get_pt(&self, use_paddr: bool) -> PageTable {
        let ptr: *const usize = if use_paddr {
            self.0.into()
        } else {
            let vaddr: KernelVAddress = self.0.into();
            vaddr.into()
        };
        PageTable(PhysAddr::new((*(ptr) << 2) & !0xfff))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageTable(PhysAddr);

impl PageTable {
    pub const fn init() -> Self {
        Self(PhysAddr::new(0))
    }
    #[inline]
    pub unsafe fn get_pte(&self, vpn: usize) -> PageTableEntry {
        let addr: *const usize = self.0.into();
        let addr = addr.add(vpn);
        PageTableEntry(PhysAddr::new(addr as usize))
    }

    pub unsafe fn activate(&self) {
        let p: PhysAddr = self.0.into();
        asm!(
            "sfence.vma x0, x0",
            "csrw satp, {satp}",
            "sfence.vma x0, x0",
            satp = in(reg) (SATP_SV48 | (p.addr >> 12))
        );
    }
}

pub unsafe fn walk(
    page_table: PageTable,
    vaddr: VirtAddr,
    alloc: bool,
    use_paddr: bool
) -> KernelResult<PageTableEntry> {
    let level = 3;
    _walk(page_table, vaddr, alloc, level, use_paddr)
}

unsafe fn _walk(
    page_table: PageTable,
    vaddr: VirtAddr,
    alloc: bool,
    level: usize,
    use_paddr: bool
) -> KernelResult<PageTableEntry> {
    let vpn = vaddr.get_vpn(level);
    let mut pte = page_table.get_pte(vpn);
    if level == 0 {
        Ok(pte)
    } else {
        if !pte.is_valid(use_paddr) {
            if !alloc {
                return Err(Err::PteNotFound);
            } else {
                let addr = alloc_pages(1)?.addr;
                // TODO: use physical address
                pte.write(((addr >> 12) << 10) | PAGE_V, use_paddr);
            }
        };
        _walk(pte.get_pt(use_paddr), vaddr, alloc, level - 1, use_paddr)
    }
}

pub unsafe fn map_page(
    root_table: PageTable,
    vaddr: VirtAddr,
    paddr: PhysAddr,
    flags: usize,
    use_paddr: bool
) -> KernelResult<()> {
    assert!(is_aligned(vaddr.addr, PAGE_SIZE), "{:?}", vaddr);
    assert!(is_aligned(paddr.addr, PAGE_SIZE));
    let mut pte = walk(root_table, vaddr, true, use_paddr)?;
    if pte.is_valid(use_paddr) {
        println!("wow");
    }
    pte.write(((paddr.addr >> 12) << 10) | flags | PAGE_V, use_paddr);
    Ok(())
}

pub unsafe fn map_pages(
    root_table: PageTable,
    vaddr: VirtAddr,
    paddr: PhysAddr,
    size: usize,
    flags: usize,
) -> KernelResult<()> {
    let mut offset = 0;
    while offset < size {
        map_page(
            root_table,
            vaddr + offset.into(),
            paddr + offset.into(),
            flags,
            true
        )?;
        offset += PAGE_SIZE
    }
    Ok(())
}

pub unsafe fn allocate_page_table() -> KernelResult<PageTable> {
    let pt = PageTable(alloc_pages(1)?);
    let k_v: KernelVAddress = KERNEL_VM.0.into();
    let new_pt_v:KernelVAddress = pt.0.into();
    ptr::copy::<u8>(k_v.into(), new_pt_v.into(), PAGE_SIZE);
    Ok(pt)
}

/// This function must be called only once at initialization.
/// After this function, we cannot access physical memory directly.
pub unsafe fn kernel_vm_init(free_ram_phys: usize, free_ram_end_phys: usize) -> KernelResult<()> {
    let kernel_pt = PageTable(alloc_pages(1)?);
    let kerenel_txt = ptr::addr_of!(__text) as usize;
    let kerenel_txt_end = ptr::addr_of!(__text_end) as usize;
    let ro_datat = ptr::addr_of!(__rodata) as usize;
    let ro_datat_end = ptr::addr_of!(__rodata_end) as usize;
    let kernel_data = ptr::addr_of!(__data) as usize;
    let kernel_data_end = ptr::addr_of!(__data_end) as usize;
    let free_ram_size = free_ram_end_phys - free_ram_phys;
    let free_ram_phys: PhysAddr = free_ram_phys.into();
    let free_ram_virt: KernelVAddress = free_ram_phys.into();

    // code region
    map_pages(
        kernel_pt,
        kerenel_txt.into(),
        (kerenel_txt & !KERNEL_CODE_PFX).into(),
        kerenel_txt_end - kerenel_txt,
        PAGE_R | PAGE_X,
    )?;
    map_pages(
        kernel_pt,
        ro_datat.into(),
        (ro_datat & !KERNEL_CODE_PFX).into(),
        ro_datat_end - ro_datat,
        PAGE_R,
    )?;
    map_pages(
        kernel_pt,
        kernel_data.into(),
        (kernel_data & !KERNEL_CODE_PFX).into(),
        kernel_data_end - kernel_data,
        PAGE_R | PAGE_W,
    )?;

    // physical region
    map_pages(
        kernel_pt,
        free_ram_virt.into(),
        free_ram_phys,
        free_ram_size,
        PAGE_R | PAGE_W,
    )?;

    // map_pages(kernel_pt, UART0.into(), UART0.into(), PAGE_SIZE, PAGE_R | PAGE_W)?;
    // map_pages(kernel_pt, VIRTIO0.into(), VIRTIO0.into(), PAGE_SIZE, PAGE_R | PAGE_W)
    // map_pages(kernel_pt, PLIC.into(), PLIC.into(), PLIC_SIZE, PAGE_R + PAGE_W)?;
    // map_pages(kernel_pt, CLINT.into(), CLINT.into(), CLINT_SIZE, PAGE_R | PAGE_W)?;
    // map_pages(kernel_pt, ACLINT_SSWI_PADDR.into(), ACLINT_SSWI_PADDR.into(), PAGE_SIZE, PAGE_R | PAGE_W)?;

    KERNEL_VM = kernel_pt;
    KERNEL_VM.activate();
    println!("activation finished");
    Ok(())
}
