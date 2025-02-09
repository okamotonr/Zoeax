use libzoea::caps::CNode;
use libzoea::caps::Endpoint;
use libzoea::caps::Notificaiton;
use libzoea::caps::Page;
use libzoea::caps::PageFlags;
use libzoea::caps::ThreadControlBlock;
use libzoea::caps::PageTable;
use libzoea::println;
use libzoea::syscall::map_page;
use libzoea::syscall::recv_ipc;
use libzoea::syscall::send_ipc;
use libzoea::syscall::send_signal;
use libzoea::syscall::traverse;
use libzoea::syscall::unmap_page;
use libzoea::syscall::CapabilityType;
use libzoea::syscall::untyped_retype;
use libzoea::BootInfo;
use libzoea::Registers;
use libzoea::shared::aligned_to::AlignedTo;

use crate::boot_info::ROOT_CNODE_RADIX;
use crate::elf::ElfProgramMapper;
use crate::boot_info::{get_untyped, get_root_cnode, get_root_vspace};


pub static mut STACK: [usize; 512] = [0; 512];


static ALIGNED: &AlignedTo<u8, [u8]> = &AlignedTo {
    _align: [],
    bytes: *include_bytes!("../simple"),
};

static SIMPLE: &[u8] = &ALIGNED.bytes;



#[no_mangle]
pub fn main(boot_info: &BootInfo) {
    // same kernel::init::root_server::ROOT_*;
    let mut root_cnode = get_root_cnode(boot_info);
    let mut untyped = get_untyped(boot_info);
    let mut root_vspace = get_root_vspace(boot_info);
    let untyped_cnode_idx = boot_info.untyped_infos[0].idx;
    println!("boot info: {:x?}", boot_info);
    println!("parent: hello, world, {untyped_cnode_idx:x}");
    let mut child_tcb = untyped.retype_single_with_fixed_size::<ThreadControlBlock>(&mut root_cnode.get_slot().unwrap()).unwrap();
    let notify = untyped.retype_single_with_fixed_size::<Notificaiton>(
        &mut root_cnode.get_slot().unwrap(),
    ).unwrap();
    let mut lv2_cnode = untyped.retype_single::<CNode>(
        &mut root_cnode.get_slot().unwrap(),
        1
    ).unwrap();

    let mut page = untyped.retype_single_with_fixed_size::<Page>(
        &mut root_cnode.get_slot().unwrap()
    ).unwrap();
    let mut page_table = untyped.retype_single_with_fixed_size::<PageTable>(
        &mut root_cnode.get_slot().unwrap()
    ).unwrap();
    let endpoint = untyped.retype_single_with_fixed_size::<Endpoint>(
        &mut root_cnode.get_slot().unwrap()
    ).unwrap();


    let vaddr = 0x0000000001000000 - 0x1000;

    page_table.map(&mut root_vspace, vaddr).unwrap();

    let flags = PageFlags::readandwrite();
    page.map(&mut root_vspace, vaddr, flags).unwrap();
    let ptr = vaddr as *mut usize;
    unsafe {
        (*ptr) = 3;
    }
    child_tcb.set_ipc_buffer(&page).unwrap();

    // test copy into lv2
    let copied_notiry = lv2_cnode.copy::<Notificaiton>(
        &notify,
    )
    .unwrap();
    let minted_not_1 = root_cnode.mint(
        &notify,
        0b100,
    )
    .unwrap();
    let minted_not_2 = root_cnode.mint(
        &notify,
        0b1000,
    )
    .unwrap();
    let minted_ep = root_cnode.mint(
        &endpoint,
        0xdeadbeef,
    )
    .unwrap();
    child_tcb.configure(
        &mut root_cnode,
        &mut root_vspace
    )
    .unwrap();

    let sp_val = unsafe {
        let stack_bottom = &mut STACK[511];
        stack_bottom as *mut usize as usize
    };
    child_tcb.write_regs(
        || Registers {
            sp: sp_val,
            sepc: children as usize,
            a0: minted_not_1.cap_ptr,
            a1: minted_ep.cap_ptr,
            a2: untyped_cnode_idx,
            ..Default::default()
        },
        boot_info.ipc_buffer(),
    )
    .unwrap();


    traverse().unwrap();
    child_tcb.resume().unwrap();
    println!("parnet: wait");
    let v = notify.wait().unwrap();
    println!("parent: wake up {v:?}");
    minted_not_2.send().unwrap();
    println!("parent: call send");
    endpoint.send().unwrap();
    println!("parnet: send done");
    println!("parent: call recv");
    endpoint.recive().unwrap();
    println!("parnet: recv done");
    println!("parent: call recv");
    endpoint.recive();
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
    unmap_page(
        page_idx,
        ROOT_CNODE_RADIX,
        root_vspace_idx,
        ROOT_CNODE_RADIX,
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
