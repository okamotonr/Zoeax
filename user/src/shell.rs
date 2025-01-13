use common::println;
use common::syscall::{untyped_retype, TYPE_TCB};

#[no_mangle]
pub fn main(untyped_cnode_idx: usize) {
    println!("hello, world, {untyped_cnode_idx:}");
    untyped_retype(untyped_cnode_idx, untyped_cnode_idx + 1, 0, 1, TYPE_TCB);
    panic!();
}
