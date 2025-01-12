use crate::{
    println,
    scheduler::{get_current_tcb, get_current_tcb_mut},
    uart::putchar,
};

use common::syscall::{PUTCHAR, RECV, SEND};

pub fn handle_syscall(syscall_n: usize) {
    match syscall_n {
        PUTCHAR => {
            let a0 = get_current_tcb().registers.a0;
            putchar(a0 as u8)
        }
        SEND => {
            println!("send called");
            panic!("Not impl")
        }
        RECV => {
            panic!("Not impl")
        }
        _ => {
            panic!("Unknown syscall, {:?}", syscall_n);
        }
    }

    // increment pc
    get_current_tcb_mut().registers.sepc += 4
}
