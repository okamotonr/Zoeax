use common::println;
use common::syscall::resume_tcb;
use common::syscall::write_reg;
use common::syscall::{configure_tcb, untyped_retype, TYPE_TCB};

pub static mut STACK: [u8; 512] = [0; 512];

#[no_mangle]
pub fn main(untyped_cnode_idx: usize) {
    // same kernel::init::root_server::ROOT_*;
    let root_cnode_idx: usize = 2;
    let root_vspace_idx: usize = 3;
    println!("hello, world, {untyped_cnode_idx:}");
    let tcb_idx = untyped_cnode_idx + 1;
    untyped_retype(untyped_cnode_idx, tcb_idx, 0, 1, TYPE_TCB);
    write_reg(tcb_idx, false, &raw mut STACK as usize);
    write_reg(tcb_idx, true, children as usize);
    configure_tcb(tcb_idx, root_cnode_idx, root_vspace_idx);
    resume_tcb(tcb_idx);
    println!("hoge");
    panic!()
}

fn children() {
    println!("hello from children");
    panic!()
}
