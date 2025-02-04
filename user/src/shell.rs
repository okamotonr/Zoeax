use common::println;
use common::syscall::cnode_copy;
use common::syscall::cnode_mint;
use common::syscall::map_page;
use common::syscall::map_page_table;
use common::syscall::recv_ipc;
use common::syscall::recv_signal;
use common::syscall::resume_tcb;
use common::syscall::send_ipc;
use common::syscall::send_signal;
use common::syscall::set_ipc_buffer;
use common::syscall::traverse;
use common::syscall::write_reg;
use common::syscall::CapabilityType;
use common::syscall::{configure_tcb, untyped_retype};
use common::BootInfo;
use common::Registers;

pub static mut STACK: [usize; 512] = [0; 512];
const ROOT_CNODE_RADIX: u32 = 18;

#[no_mangle]
pub fn main(boot_info: &BootInfo) {
    // same kernel::init::root_server::ROOT_*;
    let root_cnode_idx = boot_info.root_cnode_idx;
    let root_vspace_idx = boot_info.root_vspace_idx;
    let untyped_cnode_idx = boot_info.untyped_infos[0].idx;
    println!("boot info: {:x?}", boot_info);
    println!("parent: hello, world, {untyped_cnode_idx:x}");
    let first_empyt = boot_info.firtst_empty_idx;
    let tcb_idx = first_empyt + 1;
    untyped_retype(
        untyped_cnode_idx,
        ROOT_CNODE_RADIX,
        root_cnode_idx,
        ROOT_CNODE_RADIX,
        tcb_idx as u32,
        0,
        1,
        CapabilityType::Tcb,
    )
    .unwrap();
    let notify_idx = tcb_idx + 1;
    untyped_retype(
        untyped_cnode_idx,
        ROOT_CNODE_RADIX,
        root_cnode_idx,
        ROOT_CNODE_RADIX,
        notify_idx as u32,
        0,
        1,
        CapabilityType::Notification,
    )
    .unwrap();
    let lv2_cnode_idx = notify_idx + 1;
    untyped_retype(
        untyped_cnode_idx,
        ROOT_CNODE_RADIX,
        root_cnode_idx,
        ROOT_CNODE_RADIX,
        lv2_cnode_idx as u32,
        1,
        1,
        CapabilityType::CNode,
    )
    .unwrap();

    let page_idx = lv2_cnode_idx + 1;
    untyped_retype(
        untyped_cnode_idx,
        ROOT_CNODE_RADIX,
        root_cnode_idx,
        ROOT_CNODE_RADIX,
        page_idx as u32,
        0,
        1,
        CapabilityType::Page,
    )
    .unwrap();
    let page_table_idx = page_idx + 1;
    untyped_retype(
        untyped_cnode_idx,
        ROOT_CNODE_RADIX,
        root_cnode_idx,
        ROOT_CNODE_RADIX,
        page_table_idx as u32,
        0,
        1,
        CapabilityType::PageTable,
    )
    .unwrap();
    let ep_idx = page_table_idx + 3;
    let ep_mint_idx = page_table_idx + 4;
    untyped_retype(
        untyped_cnode_idx,
        ROOT_CNODE_RADIX,
        root_cnode_idx,
        ROOT_CNODE_RADIX,
        ep_idx as u32,
        0,
        1,
        CapabilityType::EndPoint,
    )
    .unwrap();

    let vaddr = 0x0000000001000000 - 0x1000;
    map_page_table(
        page_table_idx,
        ROOT_CNODE_RADIX,
        root_vspace_idx,
        ROOT_CNODE_RADIX,
        vaddr,
    )
    .unwrap();

    let page_r = 2;
    let page_w = 4;
    let flags = page_r | page_w;
    map_page(
        page_idx,
        ROOT_CNODE_RADIX,
        root_vspace_idx,
        ROOT_CNODE_RADIX,
        vaddr,
        flags,
    )
    .unwrap();
    let ptr = vaddr as *mut usize;
    unsafe {
        (*ptr) = 3;
    }
    set_ipc_buffer(tcb_idx, ROOT_CNODE_RADIX, page_idx, ROOT_CNODE_RADIX).unwrap();

    let sp_val = unsafe {
        let stack_bottom = &mut STACK[511];
        stack_bottom as *mut usize as usize
    };
    write_reg(
        tcb_idx,
        ROOT_CNODE_RADIX,
        || Registers {
            sp: sp_val,
            sepc: children as usize,
            a0: page_table_idx + 1,
            a1: ep_mint_idx,
            a2: untyped_cnode_idx,
            ..Default::default()
        },
        boot_info.ipc_buffer(),
    )
    .unwrap();

    let dest_idx = lv2_cnode_idx << 1;
    cnode_copy(
        root_cnode_idx,
        ROOT_CNODE_RADIX,
        dest_idx,
        ROOT_CNODE_RADIX + 1,
        notify_idx,
        ROOT_CNODE_RADIX,
    )
    .unwrap();
    cnode_mint(
        root_cnode_idx,
        ROOT_CNODE_RADIX,
        page_table_idx + 1,
        ROOT_CNODE_RADIX,
        notify_idx,
        ROOT_CNODE_RADIX,
        0b100,
    )
    .unwrap();
    cnode_mint(
        root_cnode_idx,
        ROOT_CNODE_RADIX,
        page_table_idx + 2,
        ROOT_CNODE_RADIX,
        notify_idx,
        ROOT_CNODE_RADIX,
        0b1000,
    )
    .unwrap();
    cnode_mint(
        root_cnode_idx,
        ROOT_CNODE_RADIX,
        ep_mint_idx,
        ROOT_CNODE_RADIX,
        ep_idx,
        ROOT_CNODE_RADIX,
        0xdeadbeef,
    )
    .unwrap();
    configure_tcb(
        tcb_idx,
        ROOT_CNODE_RADIX,
        root_cnode_idx,
        ROOT_CNODE_RADIX,
        root_vspace_idx,
        ROOT_CNODE_RADIX,
    )
    .unwrap();
    traverse().unwrap();
    resume_tcb(tcb_idx, ROOT_CNODE_RADIX).unwrap();
    println!("parnet: wait");
    let v = recv_signal(notify_idx, ROOT_CNODE_RADIX).unwrap();
    println!("parent: wake up {v:?}");
    send_signal(page_table_idx + 2, ROOT_CNODE_RADIX).unwrap();
    println!("parent: call send");
    send_ipc(ep_idx, ROOT_CNODE_RADIX).unwrap();
    println!("parnet: send done");
    println!("parent: call recv");
    recv_ipc(ep_idx, ROOT_CNODE_RADIX).unwrap();
    println!("parnet: recv done");
    println!("parent: call recv");
    recv_ipc(ep_idx, ROOT_CNODE_RADIX).unwrap();
    println!("parnet: recv done");
    panic!("iam parent");
}

#[allow(clippy::empty_loop)]
fn children(a0: usize, a1: usize, a2: usize) {
    let root_cnode_idx: usize = 2;
    let root_vspace_idx: usize = 3;
    println!("children: hello from children");
    println!("children: a0 is {a0}");
    println!("children: a1 is {a1}");
    println!("children: a2 is {a2}");
    send_signal(a0, ROOT_CNODE_RADIX).unwrap();
    println!("children: send signal");
    let vaddr = 0x0000000001000000 - 0x2000;
    let page_idx = a1 + 1;
    let page_r = 2;
    let page_w = 4;
    let flags = page_r | page_w;
    untyped_retype(
        a2,
        ROOT_CNODE_RADIX,
        root_cnode_idx,
        ROOT_CNODE_RADIX,
        page_idx as u32,
        1,
        1,
        CapabilityType::Page,
    )
    .unwrap();
    map_page(
        page_idx,
        ROOT_CNODE_RADIX,
        root_vspace_idx,
        ROOT_CNODE_RADIX,
        vaddr,
        flags,
    )
    .unwrap();
    println!("child: call recv");
    recv_ipc(a1, ROOT_CNODE_RADIX).unwrap();
    println!("child: recv done");
    println!("child: call send");
    send_ipc(a1, ROOT_CNODE_RADIX).unwrap();
    println!("child: send done");
    println!("child: call send");
    send_ipc(a1, ROOT_CNODE_RADIX).unwrap();
    println!("child: send done");
    panic!("iam child");
}
