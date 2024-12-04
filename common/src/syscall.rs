use core::{arch::asm, usize};
#[repr(usize)] 
pub enum SysNo {
    PutChar = PUTCHAR,
    Sleep = SLEEP,
}

pub const PUTCHAR: usize = 0;
pub const SLEEP: usize = 1;


#[no_mangle]
unsafe fn syscall(sysno: SysNo, arg0: usize, arg1: usize, arg2: usize) -> isize {
    let mut result: isize;

    asm!(
        "ecall",
        in("a0") arg0,
        in("a1") arg1,
        in("a2") arg2,
        in("a3") sysno as usize,
        lateout("a0") result,
    );

    result
}

#[no_mangle]
pub fn put_char(char: char) {

    unsafe {
        syscall(SysNo::PutChar, char as usize, 0, 0);
    }
}

pub fn sleep(ms_time: usize) {
    unsafe {
        syscall(SysNo::Sleep, ms_time, 0, 0);
    }
}
