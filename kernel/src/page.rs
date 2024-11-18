/*
 Implement sv48;

 47        39  38       30  29       21  20       12  11               0
|   VPN[3]   |   VPN[2]   |   VPN[1]   |   VPN[0]   |    page offset    | 
     9            9            9            9                12

 55                39  38       30  29       21  20       12  11               0
|       PPN[3]       |   PPN[2]   |   PPN[1]   |   PPN[0]   |    page offset    |
        17                9            9            9                12


 53                37  36       28  27       19  18       10
|       PPN[3]       |   PPN[2]   |   PPN[1]   |   PPN[0]   |
        17                9            9            9       
9    8  7 6 5 4 3 2 1 0
| RSW |D|A|G|U|X|W|R|V|
 *
 *
 */

use crate::{memory::{PhysAddr, VirtAddr, PAGE_SIZE, alloc_pages}, common::is_aligned};

/* 64bit arch*/
pub const SATP_SV48: usize = (9 << 60);
pub const PAGE_V: usize = 1 << 0;
pub const PAGE_R: usize = 1 << 1;
pub const PAGE_W: usize = 1 << 2;
pub const PAGE_X: usize = 1 << 3;
pub const PAGE_U: usize = 1 << 4;

pub fn map_page(table3: PhysAddr, vaddr: VirtAddr, paddr: PhysAddr, flags: usize){
    assert!(is_aligned(vaddr.addr, PAGE_SIZE));
    assert!(is_aligned(paddr.addr, PAGE_SIZE));

    let table3 = table3.addr as *mut usize;
    let vpn3 = vaddr.addr >> 39 & 0x1ff;
    let vpn2 = vaddr.addr >> 30 & 0x1ff; 
    let vpn1 = (vaddr.addr >> 21) & 0x1ff;
    let vpn0 = (vaddr.addr >> 12) & 0x1ff;
    unsafe {
        if ((*table3.add(vpn3)) & PAGE_V) == 0 {
            let pt_addr = alloc_pages(1);
            *table3.add(vpn3) = ((pt_addr.addr / PAGE_SIZE) << 10) | PAGE_V;
        }
    }

    let table2 = unsafe {
        let pte = table3.add(vpn3);
        let table2 = ((*pte << 2) & !0xfff) as *mut usize;
        if (*table2.add(vpn2) & PAGE_V) == 0 {
            let pt_addr = alloc_pages(1);
            *table2.add(vpn2) = ((pt_addr.addr / PAGE_SIZE) << 10) | PAGE_V;
        };
        table2
    };

    let table1 = unsafe {
        let pte = table2.add(vpn2);
        let table1 = ((*pte << 2) & !0xfff) as *mut usize;
        if (*table1.add(vpn1) & PAGE_V) == 0 {
            let pt_addr = alloc_pages(1);
            *table1.add(vpn1) = ((pt_addr.addr / PAGE_SIZE) << 10) | PAGE_V;
        };
        table1
    };

    unsafe {
        let pte = table1.add(vpn1);
        let table0 = ((*pte << 2) & !0xfff) as *mut usize;
        *table0.add(vpn0) = ((paddr.addr >> 12) << 10) | flags | PAGE_V;
    }
}

