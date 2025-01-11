use crate::address::{PhysAddr, VirtAddr};
use crate::object::PageTable;
use crate::vm::{KERNEL_VM_ROOT, LV2TABLE, KernelVAddress, KERNEL_CODE_PFX, SATP_SV48};
use crate::common::align_down;
use crate::println;

use core::ptr;
use core::arch::asm;

// so dirty...
fn pte(paddr: PhysAddr, leaf: bool) -> usize {
    let ppn = paddr.addr >> 12;
    let bottom_10 = if leaf {
        //    swdaguxwrv
        0b0011101111
    } else {
        //    swdaguxwrv
        0b0000000001
    };
    (ppn << 10) | bottom_10
}

extern "C" {
    static __text: u8;
    static __text_end: u8;
    static __data: u8;
    static __data_end: u8;
    static __rodata: u8;
    static __rodata_end: u8;
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
        asm!(
            "sfence.vma x0, x0",
            "csrw satp, {satp}",
            "sfence.vma x0, x0",
            satp = in(reg) (SATP_SV48 | (address >> 12))
        )
    }
    println!("root vm activation finished");
}
