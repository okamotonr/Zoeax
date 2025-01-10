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
// use crate::memlayout::{ACLINT_SSWI_PADDR, CLINT, CLINT_SIZE, PLIC, PLIC_SIZE, UART0};
use crate::address::{Address, PhysAddr, VirtAddr};
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
pub static mut LV2TABLE: [usize; 512] = [0; 512];

