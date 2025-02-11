use libzoea::caps::CNode;
use libzoea::caps::Endpoint;
use libzoea::caps::EndpointCapability;
use libzoea::caps::Notificaiton;
use libzoea::caps::NotificaitonCapability;
use libzoea::caps::Page;
use libzoea::caps::PageFlags;
use libzoea::caps::PageTable;
use libzoea::caps::ThreadControlBlock;
use libzoea::println;
use libzoea::shared::aligned_to::AlignedTo;
use libzoea::shared::elf::def::Elf64Hdr;
use libzoea::syscall::traverse;
use libzoea::BootInfo;
use libzoea::Registers;

use crate::boot_info::{get_root_cnode, get_root_vspace, get_untyped};
use crate::elf::ElfProgramMapper;

pub static mut STACK: [usize; 512] = [0; 512];

static ALIGNED: &AlignedTo<u8, [u8]> = &AlignedTo {
    _align: [],
    bytes: *include_bytes!("../simple"),
};

static SIMPLE: &[u8] = &ALIGNED.bytes;

fn new_proc_elf() -> &'static Elf64Hdr {
    unsafe { (SIMPLE as *const [u8] as *const Elf64Hdr).as_ref().unwrap() }
}

#[no_mangle]
pub fn main(boot_info: &BootInfo) {
    // same kernel::init::root_server::ROOT_*;
    let mut root_cnode = get_root_cnode(boot_info);
    let mut untyped = get_untyped(boot_info);
    let mut root_vspace = get_root_vspace(boot_info);
    println!("boot info: {:x?}", boot_info);
    let mut child_tcb = untyped
        .retype_single_with_fixed_size::<ThreadControlBlock>(&mut root_cnode.get_slot().unwrap())
        .unwrap();
    let notify = untyped
        .retype_single_with_fixed_size::<Notificaiton>(&mut root_cnode.get_slot().unwrap())
        .unwrap();

    let mut lv2_cnode = untyped
        .retype_single::<CNode>(&mut root_cnode.get_slot().unwrap(), 18)
        .unwrap();

    let mut root_vspace_for_new_proc = untyped
        .retype_single_with_fixed_size::<PageTable>(&mut lv2_cnode.get_slot().unwrap())
        .unwrap();

    let mut page = untyped
        .retype_single_with_fixed_size::<Page>(&mut root_cnode.get_slot().unwrap())
        .unwrap();
    let mut page_table = untyped
        .retype_single_with_fixed_size::<PageTable>(&mut root_cnode.get_slot().unwrap())
        .unwrap();
    let endpoint = untyped
        .retype_single_with_fixed_size::<Endpoint>(&mut root_cnode.get_slot().unwrap())
        .unwrap();

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
    let mut page = untyped
        .retype_single_with_fixed_size::<Page>(&mut root_cnode.get_slot().unwrap())
        .unwrap();

    let flags = PageFlags::readonly();
    page.map(&mut root_vspace, vaddr, flags).unwrap();
    page.unmap(&mut root_vspace).unwrap();

    root_vspace_for_new_proc.make_as_root().unwrap();
    let mut elf_mapper =
        ElfProgramMapper::try_new(lv2_cnode, untyped, &mut root_vspace, vaddr).unwrap();

    let new_proc_elf = new_proc_elf();
    let new_entry = new_proc_elf.e_entry;
    new_proc_elf.map_self(&mut elf_mapper).unwrap();
    println!("mapping was done");
    let (mut lv2_cnode, mut untyped, mut root_vspace_for_new_proc) = elf_mapper.finalize();

    let mut new_proc = untyped
        .retype_single_with_fixed_size::<ThreadControlBlock>(&mut lv2_cnode.get_slot().unwrap())
        .unwrap();

    // test copy into lv2
    let _copied_notiry = lv2_cnode.copy::<Notificaiton>(&notify).unwrap();
    let minted_not_1 = root_cnode.mint(&notify, 0b100).unwrap();
    let minted_not_2 = root_cnode.mint(&notify, 0b1000).unwrap();
    let minted_ep = root_cnode.mint(&endpoint, 0xdeadbeef).unwrap();
    child_tcb
        .configure(&mut root_cnode, &mut root_vspace)
        .unwrap();

    let sp_val = unsafe {
        let stack_bottom = &mut STACK[511];
        stack_bottom as *mut usize as usize
    };
    child_tcb
        .write_regs(
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
    new_proc
        .configure(&mut lv2_cnode, &mut root_vspace_for_new_proc)
        .unwrap();
    new_proc
        .write_regs(
            || Registers {
                sepc: new_entry,
                ..Default::default()
            },
            boot_info.ipc_buffer(),
        )
        .unwrap();
    new_proc.resume().unwrap();
    panic!("iam parent");
}

#[allow(clippy::empty_loop)]
fn children(not_cptr: usize, not_depth: u32, ep_ptr: usize, ep_depth: u32) {
    let not = NotificaitonCapability {
        cap_ptr: not_cptr,
        cap_depth: not_depth,
        cap_data: Notificaiton {},
    };
    let ep = EndpointCapability {
        cap_ptr: ep_ptr,
        cap_depth: ep_depth,
        cap_data: Endpoint {},
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
