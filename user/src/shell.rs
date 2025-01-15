use common::println;
use common::syscall::recv_signal;
use common::syscall::resume_tcb;
use common::syscall::send_signal;
use common::syscall::write_reg;
use common::syscall::TYPE_NOTIFY;
use common::syscall::{configure_tcb, untyped_retype, TYPE_TCB};

pub static mut STACK: [usize; 512] = [0; 512];

#[no_mangle]
pub fn main(untyped_cnode_idx: usize) {
    // same kernel::init::root_server::ROOT_*;
    let root_cnode_idx: usize = 2;
    let root_vspace_idx: usize = 3;
    println!("parent: hello, world, {untyped_cnode_idx:}");
    let tcb_idx = untyped_cnode_idx + 1;
    untyped_retype(untyped_cnode_idx, tcb_idx, 0, 1, TYPE_TCB);
    let notify_idx = tcb_idx + 1;
    untyped_retype(untyped_cnode_idx, notify_idx, 0, 1, TYPE_NOTIFY);

    let sp_val = unsafe {
        let stack_bottom = &mut STACK[511];
        stack_bottom as *mut usize as usize
    };
    write_reg(tcb_idx, 0, sp_val);
    write_reg(tcb_idx, 1, children as usize);
    write_reg(tcb_idx, 2, notify_idx);
    configure_tcb(tcb_idx, root_cnode_idx, root_vspace_idx);
    resume_tcb(tcb_idx);
    println!("parnet: wait");
    let v = recv_signal(notify_idx);
    println!("parent: wake up {v}");
    send_signal(notify_idx);
    panic!()
}

#[allow(clippy::empty_loop)]
fn children(a0: usize) {
    println!("children: hello from children");
    println!("children: a0 is {a0}");
    send_signal(a0);
    println!("children: send signal");
    recv_signal(a0);
    println!("child: wake up");
    loop {}
}
