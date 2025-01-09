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

use core::arch::asm;
use core::ptr;

use crate::common::{align_down, align_up, is_aligned, KernelResult};
// use crate::memlayout::{ACLINT_SSWI_PADDR, CLINT, CLINT_SIZE, PLIC, PLIC_SIZE, UART0};
use crate::memory::{Address, PhysAddr, VirtAddr};
use crate::object::PageTable;
use crate::println;
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

impl Into<PhysAddr> for VirtAddr {
    fn into(self) -> PhysAddr {
        PhysAddr::new(self.addr)
    }
}

impl VirtAddr {
    #[inline]
    pub fn get_vpn(&self, idx: usize) -> usize {
        self.addr >> (12 + idx * 9) & 0x1ff
    }
}

#[link_section = "__kernel_vm_root"]
pub static mut KERNEL_VM_ROOT: [usize; 512] = [0; 512];
#[link_section = "__lv2table"]
static mut LV2TABLE: [usize; 512] = [0; 512];

// so dirty...
fn pte(paddr: PhysAddr, leaf: bool) -> usize {
    let ppn = (paddr.addr >> 12);
    let bottom_10 = if leaf {
        //    swdaguxwrv
        0b0011101111
    } else {
        //    swdaguxwrv
        0b0000000001
    };
    ppn << 10 | bottom_10
}

/// This function must be called only once at initialization.
/// After this function, we cannot access physical memory directly.
pub unsafe fn kernel_vm_init(free_ram_end_phys: usize) {
    let phyisical_start: usize = 0;
    let phyisical_end: usize = free_ram_end_phys;
    let kerenel_txt = ptr::addr_of!(__text) as usize;
    let kernel_data_end = ptr::addr_of!(__data_end) as usize;

    // first, mappin all physical memory
    let step: usize = 2_usize.pow(9 + 9 + 9 + 12);
    for paddr in (phyisical_start..phyisical_end).step_by(step) {
        let paddr: PhysAddr = paddr.into();
        let vaddr: VirtAddr = KernelVAddress::from(paddr).into();
        let vpn = vaddr.get_vpn(3);
        KERNEL_VM_ROOT[vpn] = pte(paddr, true);
    }
    let vpn = VirtAddr::from(kerenel_txt).get_vpn(3);
    let lv2_addr = PhysAddr::from(&raw const LV2TABLE);
    KERNEL_VM_ROOT[vpn] = pte(lv2_addr.bit_and(!KERNEL_CODE_PFX), false);
    // mapping elf;
    let step = 2_usize.pow(9 + 9 + 12);
    let elf_v_start = align_down(kerenel_txt, step);
    for vaddr in (elf_v_start..kernel_data_end).step_by(step) {
        let vaddr: VirtAddr = vaddr.into();
        let paddr: PhysAddr = vaddr.bit_and(!KERNEL_CODE_PFX).into();
        let vpn = vaddr.get_vpn(2);
        LV2TABLE[vpn] = pte(paddr, true)
    }

    {
        let address = &raw const KERNEL_VM_ROOT as *const PageTable as usize;
        let address = address & !KERNEL_CODE_PFX;
        println!("{:x}", address);
        asm!(
            "sfence.vma x0, x0",
            "csrw satp, {satp}",
            "sfence.vma x0, x0",
            satp = in(reg) (SATP_SV48 | (address >> 12))
        )
    }
    println!("root vm activation finished");
}
