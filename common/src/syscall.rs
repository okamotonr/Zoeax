use core::{arch::asm, usize};
#[repr(usize)] 
pub enum SysNo {
    PutChar = PUTCHAR,
    Sleep = SLEEP,
    Send = SEND,
    Recv = RECV,
}

pub const PUTCHAR: usize = 0;
pub const SLEEP: usize = 1;
pub const SEND: usize = 2;
pub const RECV: usize = 3;

#[derive(Clone, Copy, Debug)]
pub struct Message {
    pub tag: isize,
    pub src: usize, // proccess id, who send this message.
    pub data: [u8; 1024],
}

impl Message {
    pub fn new() -> Self {
        Self {tag: 0, src: 0, data: [0; 1024]}
    }
}

unsafe fn syscall(sysno: SysNo, arg0: usize, arg1: usize, arg2: usize, arg3: usize) -> isize {
    let mut result: isize;

    asm!(
        "ecall",
        in("a0") arg0,
        in("a1") arg1,
        in("a2") arg2,
        in("a3") arg3,
        in("a4") sysno as usize,
        lateout("a0") result,
    );

    result
}

pub fn put_char(char: char) {

    unsafe {
        syscall(SysNo::PutChar, char as usize, 0, 0, 0);
    }
}

pub fn sleep(ms_time: usize) {
    unsafe {
        syscall(SysNo::Sleep, ms_time, 0, 0, 0);
    }
}

pub fn send(dst: usize, sm: &Message) {
    unsafe {
        let sm = sm as *const Message as usize;
        syscall(SysNo::Send, dst, sm, 0, 0);
    }
}
pub fn recieve(rcv: &mut Message) {
    unsafe {
        let rm = rcv as *mut Message as usize;
        syscall(SysNo::Recv, rm, 0, 0, 0);
    }

}
