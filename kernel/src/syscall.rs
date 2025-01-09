use crate::{
    common::{Err, KernelResult},
    memory::{copy_from_user, copy_to_user, VirtAddr},
    println,
    scheduler::{CURRENT_PROC, SCHEDULER},
    uart::putchar,
};

use core::ptr;

use common::syscall::{Message, PUTCHAR, RECV, SEND, SLEEP};

pub fn handle_syscall(a0: usize, a1: usize, _a2: usize, _a3: usize, syscall_n: usize) {
    match syscall_n {
        PUTCHAR => putchar(a0 as u8),
        SLEEP => {
            panic!("Not impl")
        }
        SEND => {
            println!("send called");
            println!("arg0 {}, arg1 {:x}", a0, a1);
            panic!("Not impl")
        }
        RECV => {
            panic!("Not impl")
        }
        _ => {
            panic!("Unknown syscall, {:?}", syscall_n);
        }
    }
}

