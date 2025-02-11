use libzoea::caps::CNode;
use libzoea::caps::Endpoint;
use libzoea::caps::EndpointCapability;
use libzoea::caps::Notificaiton;
use libzoea::caps::NotificaitonCapability;
use libzoea::caps::Page;
use libzoea::caps::PageFlags;
use libzoea::caps::ThreadControlBlock;
use libzoea::caps::PageTable;
use libzoea::println;
use libzoea::syscall::traverse;
use libzoea::BootInfo;
use libzoea::Registers;
use libzoea::shared::aligned_to::AlignedTo;

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
    println!("boot info: {:x?}", boot_info);
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


    let vaddr = 0x0000000001000000 - 0x2000;
    let mut page = untyped.retype_single_with_fixed_size::<Page>(
        &mut root_cnode.get_slot().unwrap()
    ).unwrap();

    let flags = PageFlags::readonly();
    page.map(&mut root_vspace, vaddr, flags).unwrap();
    // test copy into lv2
    page.unmap(&mut root_vspace).unwrap();
    let _copied_notiry = lv2_cnode.copy::<Notificaiton>(
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
            a1: minted_not_1.cap_depth as usize,
            a2: minted_ep.cap_ptr,
            a3: minted_ep.cap_depth as usize,
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
    endpoint.recive().unwrap();
    println!("parnet: recv done");
    panic!("iam parent");
}

#[allow(clippy::empty_loop)]
fn children(not_cptr: usize, not_depth: u32, ep_ptr: usize, ep_depth: u32) {
    let not = NotificaitonCapability {
        cap_ptr: not_cptr,
        cap_depth: not_depth,
        cap_data: Notificaiton {  }
    };
    let ep = EndpointCapability {
        cap_ptr: ep_ptr,
        cap_depth: ep_depth,
        cap_data: Endpoint {}
    };
    println!("children: notification is {not:?}");
    println!("children: ep is {ep:?}");
    not.send().unwrap();
    println!("children: send signal");
    println!("child: call recv");
    ep.recive().unwrap();
    println!("child: recv done");
    println!("child: call send");
    ep.send().unwrap();
    println!("child: send done");
    println!("child: call send");
    ep.send().unwrap();
    println!("child: send done");
    panic!("iam child");
}
