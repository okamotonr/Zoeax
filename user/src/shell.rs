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
use common::syscall::write_reg;
use common::syscall::CapabilityType;
use common::syscall::{configure_tcb, untyped_retype};

pub static mut STACK: [usize; 512] = [0; 512];
const ROOT_CNODE_RADIX: u32 = 18;

#[no_mangle]
pub fn main(untyped_cnode_idx: usize) {
    // same kernel::init::root_server::ROOT_*;
    let root_cnode_idx: usize = 2;
    let root_vspace_idx: usize = 3;
    println!("parent: hello, world, {untyped_cnode_idx:}");
    let tcb_idx = untyped_cnode_idx + 1;
    untyped_retype(untyped_cnode_idx, tcb_idx, 0, 1, CapabilityType::Tcb).unwrap();
    let notify_idx = tcb_idx + 1;
    untyped_retype(
        untyped_cnode_idx,
        notify_idx,
        0,
        1,
        CapabilityType::Notification,
    )
    .unwrap();
    let lv2_cnode_idx = notify_idx + 1;
    untyped_retype(
        untyped_cnode_idx,
        lv2_cnode_idx,
        1,
        1,
        CapabilityType::CNode,
    )
    .unwrap();

    let page_idx = lv2_cnode_idx + 1;
    untyped_retype(untyped_cnode_idx, page_idx, 0, 1, CapabilityType::Page).unwrap();
    let page_table_idx = page_idx + 1;
    untyped_retype(
        untyped_cnode_idx,
        page_table_idx,
        0,
        1,
        CapabilityType::PageTable,
    )
    .unwrap();
    let ep_idx = page_table_idx + 3;
    let ep_mint_idx = page_table_idx + 4;
    untyped_retype(untyped_cnode_idx, ep_idx, 0, 1, CapabilityType::EndPoint).unwrap();

    let vaddr = 0x0000000001000000 - 0x1000;
    map_page_table(page_table_idx, root_vspace_idx, vaddr).unwrap();

    let page_r = 2;
    let page_w = 4;
    let flags = page_r | page_w;
    map_page(page_idx, root_vspace_idx, vaddr, flags).unwrap();
    let ptr = vaddr as *mut usize;
    unsafe {
        (*ptr) = 3;
    }
    set_ipc_buffer(tcb_idx, page_idx).unwrap();

    let sp_val = unsafe {
        let stack_bottom = &mut STACK[511];
        stack_bottom as *mut usize as usize
    };
    write_reg(tcb_idx, 0, sp_val).unwrap();
    write_reg(tcb_idx, 1, children as usize).unwrap();

    let dest_idx = lv2_cnode_idx << 1;
    println!("heyheyheyhey     copy");
    println!("{:#b}", dest_idx);
    println!("{:#b}", lv2_cnode_idx);
    cnode_copy(
        root_cnode_idx,
        notify_idx,
        ROOT_CNODE_RADIX,
        root_cnode_idx,
        dest_idx,
        ROOT_CNODE_RADIX + 1,
    )
    .unwrap();

    cnode_mint(
        root_cnode_idx,
        notify_idx,
        ROOT_CNODE_RADIX,
        root_cnode_idx,
        page_table_idx + 1,
        ROOT_CNODE_RADIX,
        0b100,
    )
    .unwrap();
    cnode_mint(
        root_cnode_idx,
        notify_idx,
        ROOT_CNODE_RADIX,
        root_cnode_idx,
        page_table_idx + 2,
        ROOT_CNODE_RADIX,
        0b1000,
    )
    .unwrap();
    cnode_mint(
        root_cnode_idx,
        ep_idx,
        ROOT_CNODE_RADIX,
        root_cnode_idx,
        ep_mint_idx,
        ROOT_CNODE_RADIX,
        0xdeadbeef,
    )
    .unwrap();
    write_reg(tcb_idx, 2, page_table_idx + 1).unwrap();
    write_reg(tcb_idx, 3, ep_mint_idx).unwrap();
    write_reg(tcb_idx, 4, untyped_cnode_idx).unwrap();
    configure_tcb(tcb_idx, root_cnode_idx, root_vspace_idx).unwrap();
    resume_tcb(tcb_idx).unwrap();
    println!("parnet: wait");
    let v = recv_signal(notify_idx).unwrap();
    println!("parent: wake up {v:?}");
    send_signal(page_table_idx + 2).unwrap();
    println!("parent: call send");
    send_ipc(ep_idx).unwrap();
    println!("parnet: send done");
    println!("parent: call recv");
    recv_ipc(ep_idx).unwrap();
    println!("parnet: recv done");
    println!("parent: call recv");
    recv_ipc(ep_idx).unwrap();
    println!("parnet: recv done");
    panic!()
}

#[allow(clippy::empty_loop)]
fn children(a0: usize, a1: usize, a2: usize) {
    let _root_cnode_idx: usize = 2;
    let root_vspace_idx: usize = 3;
    println!("children: hello from children");
    println!("children: a0 is {a0}");
    println!("children: a1 is {a1}");
    println!("children: a2 is {a2}");
    send_signal(a0).unwrap();
    println!("children: send signal");
    let v = recv_signal(a0);
    println!("child: wake up {v:?}");
    let vaddr = 0x0000000001000000 - 0x2000;
    let page_idx = a1 + 1;
    let page_r = 2;
    let page_w = 4;
    let flags = page_r | page_w;
    untyped_retype(a2, page_idx, 1, 1, CapabilityType::Page).unwrap();
    map_page(page_idx, root_vspace_idx, vaddr, flags).unwrap();
    println!("child: call recv");
    recv_ipc(a1).unwrap();
    println!("child: recv done");
    println!("child: call send");
    send_ipc(a1).unwrap();
    println!("child: send done");
    println!("child: call send");
    send_ipc(a1).unwrap();
    println!("child: send done");
    loop {}
}
