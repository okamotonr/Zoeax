use common::println;
use common::syscall::{untyped_retype, TYPE_TCB};
use common::syscall::write_reg;

pub static mut STACK: [u8; 512] = [0; 512];

#[no_mangle]
pub fn main(untyped_cnode_idx: usize) {
    println!("hello, world, {untyped_cnode_idx:}");
    let tcb_idx = untyped_cnode_idx + 1;
    untyped_retype(untyped_cnode_idx, tcb_idx, 0, 1, TYPE_TCB);
    write_reg(tcb_idx, false, &raw mut STACK as usize);
    write_reg(tcb_idx, true, children as usize);
    println!("hoge");
    panic!();
}

fn children() {
    println!("hello from children")
}
