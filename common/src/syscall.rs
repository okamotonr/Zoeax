use core::arch::asm;
#[repr(usize)]
pub enum SysNo {
    PutChar = PUTCHAR,
    Call = CALL,
    Send = SEND,
    Recv = RECV,
}

pub const PUTCHAR: usize = 0;
pub const CALL: usize = 1;
pub const SEND: usize = 2;
pub const RECV: usize = 3;

/// inv label
pub const UNTYPED_RETYPE: usize = 1;
pub const TCB_CONFIGURE: usize = 2;
pub const TCB_WRITE_REG: usize = 3;
pub const TCB_RESUME: usize = 4;
pub const NOTIFY_WAIT: usize = 5;
pub const NOTIFY_SEND: usize = 6;

// TODO: same kernel::capability::CapabilityType
pub const TYPE_TCB: usize = 3;
pub const TYPE_NOTIFY: usize = 9;

#[allow(clippy::too_many_arguments)]
unsafe fn syscall(
    src_ptr: usize,
    inv_label: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
    sysno: SysNo,
) -> isize {
    let mut result: isize;

    asm!(
        "ecall",
        in("a0") src_ptr,
        in("a1") inv_label,
        in("a2") arg2,
        in("a3") arg3,
        in("a4") arg4,
        in("a5") arg5,
        in("a6") arg6,
        in("a7") sysno as usize,
        lateout("a0") result,
    );

    result
}

pub fn put_char(char: u8) {
    unsafe {
        syscall(char as usize, 0, 0, 0, 0, 0, 0, SysNo::PutChar);
    }
}

pub fn untyped_retype(
    src_ptr: usize,
    dest_ptr: usize,
    user_size: usize,
    num: usize,
    cap_type: usize,
) {
    unsafe {
        syscall(
            src_ptr,
            UNTYPED_RETYPE,
            dest_ptr,
            user_size,
            num,
            cap_type,
            0,
            SysNo::Call,
        );
    }
}

pub fn write_reg(src_ptr: usize, is_ip: usize, value: usize) {
    unsafe {
        syscall(
            src_ptr,
            TCB_WRITE_REG,
            is_ip,
            value,
            0,
            0,
            0,
            SysNo::Call,
        );
    }
}

pub fn configure_tcb(src_ptr: usize, cnode_ptr: usize, vspace_ptr: usize) {
    unsafe {
        syscall(
            src_ptr,
            TCB_CONFIGURE,
            cnode_ptr,
            vspace_ptr,
            0,
            0,
            0,
            SysNo::Call,
        );
    }
}

pub fn resume_tcb(src_ptr: usize) {
    unsafe {
        syscall(src_ptr, TCB_RESUME, 0, 0, 0, 0, 0, SysNo::Call);
    }
}

pub fn send_signal(src_ptr: usize) {
    unsafe {
        syscall(src_ptr, NOTIFY_SEND, 0, 0, 0, 0, 0, SysNo::Send);
    }
}

pub fn recv_signal(src_ptr: usize) -> usize {
    unsafe {
        syscall(src_ptr, NOTIFY_WAIT, 0, 0, 0, 0, 0, SysNo::Recv);
    }
    src_ptr // TODO: return a0
}
