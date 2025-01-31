use core::arch::asm;
use kernel::kerr;
use kernel::ErrKind;
use kernel::InvLabel;
use kernel::KernelResult;
use kernel::SysCallNo;

pub use kernel::CapabilityType;
// TODO: use kernel::common
pub type SysCallRes = KernelResult<usize>;

#[allow(clippy::too_many_arguments)]
unsafe fn syscall(
    src_ptr: usize,
    inv_label: InvLabel,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
    sysno: SysCallNo,
) -> SysCallRes {
    let mut is_error: usize;
    let mut val: usize;

    asm!(
        "ecall",
        inout("a0") src_ptr => is_error,
        inout("a1") inv_label as usize => val,
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
        let e_kind = ErrKind::try_from(is_error).unwrap();
        Err(kerr!(e_kind, val as u16))
    }
}

pub fn put_char(char: u8) -> SysCallRes {
    unsafe {
        syscall(
            char as usize,
            InvLabel::PutChar,
            0,
            0,
            0,
            0,
            0,
            SysCallNo::PutChar,
        )
    }
}

pub fn untyped_retype(
    src_ptr: usize,
    dest_ptr: usize,
    user_size: usize,
    num: usize,
    cap_type: CapabilityType,
) -> SysCallRes {
    unsafe {
        syscall(
            src_ptr,
            InvLabel::UntypedRetype,
            dest_ptr,
            user_size,
            num,
            cap_type as usize,
            0,
            SysCallNo::Call,
        )
    }
}

pub fn write_reg(src_ptr: usize, is_ip: usize, value: usize) -> SysCallRes {
    unsafe {
        syscall(
            src_ptr,
            InvLabel::TcbWriteReg,
            is_ip,
            value,
            0,
            0,
            0,
            SysCallNo::Call,
        )
    }
}

pub fn set_ipc_buffer(src_ptr: usize, page_cap_ptr: usize) -> SysCallRes {
    unsafe {
        syscall(
            src_ptr,
            InvLabel::TcbSetIpcBuffer,
            page_cap_ptr,
            0,
            0,
            0,
            0,
            SysCallNo::Call,
        )
    }
}

pub fn configure_tcb(src_ptr: usize, cnode_ptr: usize, vspace_ptr: usize) -> SysCallRes {
    unsafe {
        syscall(
            src_ptr,
            InvLabel::TcbConfigure,
            cnode_ptr,
            vspace_ptr,
            0,
            0,
            0,
            SysCallNo::Call,
        )
    }
}

pub fn resume_tcb(src_ptr: usize) -> SysCallRes {
    unsafe { syscall(src_ptr, InvLabel::TcbResume, 0, 0, 0, 0, 0, SysCallNo::Call) }
}

pub fn send_signal(src_ptr: usize) -> SysCallRes {
    unsafe {
        syscall(
            src_ptr,
            InvLabel::NotifySend,
            0,
            0,
            0,
            0,
            0,
            SysCallNo::Send,
        )
    }
}

pub fn recv_signal(src_ptr: usize) -> SysCallRes {
    unsafe {
        syscall(
            src_ptr,
            InvLabel::NotifyWait,
            0,
            0,
            0,
            0,
            0,
            SysCallNo::Recv,
        )
    }
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
            InvLabel::CNodeCopy,
            src_index,
            depth,
            dest_root,
            dest_index,
            0,
            SysCallNo::Call,
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
            InvLabel::CNodeMint,
            src_index,
            depth,
            dest_root,
            dest_index,
            cap_val,
            SysCallNo::Call,
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
            InvLabel::CNodeMove,
            src_index,
            depth,
            dest_root,
            dest_index,
            0,
            SysCallNo::Call,
        )
    }
}

pub fn map_page(src_ptr: usize, dest_ptr: usize, vaddr: usize, flags: usize) -> SysCallRes {
    unsafe {
        syscall(
            src_ptr,
            InvLabel::PageMap,
            dest_ptr,
            vaddr,
            flags,
            0,
            0,
            SysCallNo::Call,
        )
    }
}

pub fn map_page_table(src_ptr: usize, dest_ptr: usize, vaddr: usize) -> SysCallRes {
    unsafe {
        syscall(
            src_ptr,
            InvLabel::PageTableMap,
            dest_ptr,
            vaddr,
            0,
            0,
            0,
            SysCallNo::Call,
        )
    }
}

pub fn send_ipc(src_ptr: usize) -> SysCallRes {
    unsafe { syscall(src_ptr, InvLabel::EpSend, 0, 0, 0, 0, 0, SysCallNo::Send) }
}

pub fn recv_ipc(src_ptr: usize) -> SysCallRes {
    unsafe { syscall(src_ptr, InvLabel::EpRecv, 0, 0, 0, 0, 0, SysCallNo::Recv) }
}

pub const MESSAGE_LEN: usize = 128;

pub struct IPCBuffer {
    pub tag: usize,
    pub message: [usize; MESSAGE_LEN],
    pub user_data: usize,
}
