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
pub const TCB_SET_IPC_BUFFER: usize = 5;
pub const NOTIFY_WAIT: usize = 6;
pub const NOTIFY_SEND: usize = 7;
pub const CNODE_COPY: usize = 8;
pub const CNODE_MINT: usize = 9;
pub const CNODE_MOVE: usize = 10;
pub const PAGE_MAP: usize = 11;
pub const PAGE_TABLE_MAP: usize = 12;
pub const EP_SEND: usize = 13;
pub const EP_RECV: usize = 14;

// TODO: same kernel::capability::CapabilityType
pub const TYPE_TCB: usize = 3;
pub const TYPE_EP: usize = 5;
pub const TYPE_CNODE: usize = 7;
pub const TYPE_NOTIFY: usize = 9;
pub const TYPE_PAGE_TABLE: usize = 2;
pub const TYPE_PAGE: usize = 4;

// TODO: use kernel::common
pub type SysCallRes = Result<usize, (usize, usize)>;

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
) -> SysCallRes {
    let mut is_error: usize;
    let mut val: usize;

    asm!(
        "ecall",
        inout("a0") src_ptr => is_error,
        inout("a1") inv_label => val,
        in("a2") arg2,
        in("a3") arg3,
        in("a4") arg4,
        in("a5") arg5,
        in("a6") arg6,
        in("a7") sysno as usize,
    );

    if is_error == 0 {
        Ok(val)
    } else {
        Err((is_error, val))
    }
}

pub fn put_char(char: u8) -> SysCallRes {
    unsafe { syscall(char as usize, 0, 0, 0, 0, 0, 0, SysNo::PutChar) }
}

pub fn untyped_retype(
    src_ptr: usize,
    dest_ptr: usize,
    user_size: usize,
    num: usize,
    cap_type: usize,
) -> SysCallRes {
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
        )
    }
}

pub fn write_reg(src_ptr: usize, is_ip: usize, value: usize) -> SysCallRes {
    unsafe { syscall(src_ptr, TCB_WRITE_REG, is_ip, value, 0, 0, 0, SysNo::Call) }
}

pub fn set_ipc_buffer(src_ptr: usize, page_cap_ptr: usize) -> SysCallRes {
    unsafe {
        syscall(
            src_ptr,
            TCB_SET_IPC_BUFFER,
            page_cap_ptr,
            0,
            0,
            0,
            0,
            SysNo::Call,
        )
    }
}

pub fn configure_tcb(src_ptr: usize, cnode_ptr: usize, vspace_ptr: usize) -> SysCallRes {
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
        )
    }
}

pub fn resume_tcb(src_ptr: usize) -> SysCallRes {
    unsafe { syscall(src_ptr, TCB_RESUME, 0, 0, 0, 0, 0, SysNo::Call) }
}

pub fn send_signal(src_ptr: usize) -> SysCallRes {
    unsafe { syscall(src_ptr, NOTIFY_SEND, 0, 0, 0, 0, 0, SysNo::Send) }
}

pub fn recv_signal(src_ptr: usize) -> SysCallRes {
    unsafe { syscall(src_ptr, NOTIFY_WAIT, 0, 0, 0, 0, 0, SysNo::Recv) }
}

pub fn cnode_copy(
    src_root: usize,
    src_index: usize,
    src_depth: u32,
    dest_root: usize,
    dest_index: usize,
    dest_depth: u32,
) -> SysCallRes {
    let depth = ((src_depth as usize) << 31) | dest_depth as usize;
    unsafe {
        syscall(
            src_root,
            CNODE_COPY,
            src_index,
            depth,
            dest_root,
            dest_index,
            0,
            SysNo::Call,
        )
    }
}

pub fn cnode_mint(
    src_root: usize,
    src_index: usize,
    src_depth: u32,
    dest_root: usize,
    dest_index: usize,
    dest_depth: u32,
    cap_val: usize,
) -> SysCallRes {
    let depth = ((src_depth as usize) << 31) | dest_depth as usize;
    unsafe {
        syscall(
            src_root,
            CNODE_MINT,
            src_index,
            depth,
            dest_root,
            dest_index,
            cap_val,
            SysNo::Call,
        )
    }
}

pub fn cnode_move(
    src_root: usize,
    src_index: usize,
    src_depth: u32,
    dest_root: usize,
    dest_index: usize,
    dest_depth: u32,
) -> SysCallRes {
    let depth = ((src_depth as usize) << 31) | dest_depth as usize;
    unsafe {
        syscall(
            src_root,
            CNODE_MOVE,
            src_index,
            depth,
            dest_root,
            dest_index,
            0,
            SysNo::Call,
        )
    }
}

pub fn map_page(src_ptr: usize, dest_ptr: usize, vaddr: usize, flags: usize) -> SysCallRes {
    unsafe { syscall(src_ptr, PAGE_MAP, dest_ptr, vaddr, flags, 0, 0, SysNo::Call) }
}

pub fn map_page_table(src_ptr: usize, dest_ptr: usize, vaddr: usize) -> SysCallRes {
    unsafe {
        syscall(
            src_ptr,
            PAGE_TABLE_MAP,
            dest_ptr,
            vaddr,
            0,
            0,
            0,
            SysNo::Call,
        )
    }
}

pub fn send_ipc(src_ptr: usize) -> SysCallRes {
    unsafe { syscall(src_ptr, EP_SEND, 0, 0, 0, 0, 0, SysNo::Send) }
}

pub fn recv_ipc(src_ptr: usize) -> SysCallRes {
    unsafe { syscall(src_ptr, EP_RECV, 0, 0, 0, 0, 0, SysNo::Recv) }
}

pub const MESSAGE_LEN: usize = 128;

pub struct IPCBuffer {
    pub tag: usize,
    pub message: [usize; MESSAGE_LEN],
    pub user_data: usize,
}
