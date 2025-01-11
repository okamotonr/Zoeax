use crate::{println, uart::putchar};

use common::syscall::{PUTCHAR, RECV, SEND, SLEEP};

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
