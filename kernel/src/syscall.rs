use crate::{
    address::PAGE_SIZE,
    capability::CapabilityType,
    common::{is_aligned, ErrKind, KernelResult},
    kerr,
    object::{
        page_table::{Page, PAGE_U},
        CNode, CNodeEntry, Endpoint, Notification, PageTable, Registers, ThreadControlBlock,
        Untyped,
    },
    println,
    scheduler::{get_current_tcb_mut, require_schedule},
    uart::putchar,
    KernelError,
};

#[repr(u8)]
pub enum SysCallNo {
    PutChar = 0,
    Call = 1,
    Send = 2,
    Recv = 3,
}

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvLabel {
    PutChar = 0,
    UntypedRetype = 1,
    TcbConfigure,
    TcbWriteReg,
    TcbResume,
    TcbSetIpcBuffer,
    NotifyWait,
    NotifySend,
    CNodeCopy,
    CNodeMint,
    CNodeMove,
    PageMap,
    PageUnMap,
    PageTableMap,
    PageTableUnMap,
    EpSend,
    EpRecv,
}

impl TryFrom<usize> for InvLabel {
    type Error = KernelError;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            inv if inv == Self::PutChar as usize => Ok(Self::PutChar),
            inv if inv == Self::UntypedRetype as usize => Ok(Self::UntypedRetype),
            inv if inv == Self::TcbConfigure as usize => Ok(Self::TcbConfigure),
            inv if inv == Self::TcbWriteReg as usize => Ok(Self::TcbWriteReg),
            inv if inv == Self::TcbResume as usize => Ok(Self::TcbResume),
            inv if inv == Self::TcbSetIpcBuffer as usize => Ok(Self::TcbSetIpcBuffer),
            inv if inv == Self::NotifyWait as usize => Ok(Self::NotifyWait),
            inv if inv == Self::NotifySend as usize => Ok(Self::NotifySend),
            inv if inv == Self::CNodeCopy as usize => Ok(Self::CNodeCopy),
            inv if inv == Self::CNodeMint as usize => Ok(Self::CNodeMint),
            inv if inv == Self::CNodeMove as usize => Ok(Self::CNodeMove),
            inv if inv == Self::PageMap as usize => Ok(Self::PageMap),
            inv if inv == Self::PageUnMap as usize => Ok(Self::PageUnMap),
            inv if inv == Self::PageTableMap as usize => Ok(Self::PageTableMap),
            inv if inv == Self::PageTableUnMap as usize => Ok(Self::PageTableUnMap),
            inv if inv == Self::EpSend as usize => Ok(Self::EpSend),
            inv if inv == Self::EpRecv as usize => Ok(Self::EpRecv),
            _ => Err(kerr!(ErrKind::UnknownInvocation)),
        }
    }
}

pub fn handle_syscall(syscall_n: usize, reg: &mut Registers) {
    let cap_ptr = reg.a0;
    let depth = reg.a1;
    let inv_label = reg.a2;
    let syscall_ret = match syscall_n {
        n if n == SysCallNo::PutChar as usize => {
            let a0 = reg.a0;
            putchar(a0 as u8);
            Ok(())
        }
        _ => {
            // Why don't you use "?"?
            handle_invocation(cap_ptr, depth, inv_label, syscall_n, reg)
        }
    };
    if let Err(e) = syscall_ret {
        println!("system call failed, {:?}", e);
        reg.a0 = e.e_kind as usize;
        reg.a1 = e.e_val as usize;
    } else {
        reg.a0 = 0;
    }
    // increment pc
    reg.sepc += 4;
}

fn handle_invocation(
    cap_ptr: usize,
    depth: usize,
    inv_label: usize,
    _syscall_n: usize,
    reg: &Registers,
) -> KernelResult<()> {
    let inv_label = InvLabel::try_from(inv_label)?;
    let current_tcb = get_current_tcb_mut();
    let mut root_cnode = current_tcb.root_cnode.as_ref().unwrap().cap().replicate();
    let slot = current_tcb
        .root_cnode
        .as_mut()
        .ok_or(kerr!(ErrKind::NoMemory))?
        .cap_ref_mut()
        .lookup_entry_mut(cap_ptr, depth as u32)?
        .as_mut()
        .ok_or(kerr!(ErrKind::SlotIsEmpty))?;
    let cap_type = slot.get_cap_type()?;
    // TODO: Into Capability::invoke()
    match cap_type {
        CapabilityType::Untyped => {
            let dest_cnode_ptr = reg.a3;
            let user_size = reg.a4;
            let num = reg.a5;
            let new_type = CapabilityType::try_from_u8(reg.a6 as u8)?;
            let (_, dest_cnode) = root_cnode.get_src_and_dest(cap_ptr, dest_cnode_ptr, num)?;
            let (src_cap, src_mdb) = slot.cap_and_mdb();
            src_cap
                .as_capability::<Untyped>()?
                .invoke_retype(src_mdb, dest_cnode, user_size, num, new_type)
        }
        CapabilityType::CNode => {
            let src_index = reg.a3;
            let src_depth = (reg.a4 >> 32) as u32;
            let dest_depth = reg.a4 as u32;
            let dest_index = reg.a5;
            // TODO: get 2 ref

            let mut dest_root = slot.cap_ref_mut().as_capability::<CNode>()?.replicate();
            let mut src_root = root_cnode.replicate();
            let src_slot = src_root.lookup_entry_mut(src_index, src_depth)?;
            let src_entry = src_slot.as_mut().ok_or(kerr!(ErrKind::SlotIsEmpty))?;
            let dest_slot = dest_root.lookup_entry_mut(dest_index, dest_depth)?;
            if dest_slot.is_some() {
                Err(kerr!(ErrKind::NotEmptySlot))
            } else {
                // TODO: Whether this cap is derivable
                let raw_cap = src_entry.cap().replicate();
                let mut cap = raw_cap;
                if inv_label == InvLabel::CNodeMint {
                    let cap_val = reg.a6;
                    cap.set_cap_dep_val(cap_val);
                }
                let mut new_slot = CNodeEntry::new_with_rawcap(cap);
                if inv_label == InvLabel::CNodeMove {
                    new_slot.replace(src_entry);
                    *src_slot = None
                } else {
                    new_slot.insert(src_entry);
                }
                *dest_slot = Some(new_slot);
                Ok(())
            }
        }
        CapabilityType::Tcb => {
            let tcb_cap = slot.cap_ref_mut().as_capability::<ThreadControlBlock>()?;
            match inv_label {
                InvLabel::TcbConfigure => {
                    let cnode_ptr = reg.a3;
                    let cnode_depth = reg.a4 as u32;
                    let vspace_ptr = reg.a5;
                    let vspace_depth = reg.a6 as u32;
                    // TODO: Impl get 2 entry from cnode with safety check
                    let mut todo_root_cnode = root_cnode.replicate();
                    let cspace_slot = root_cnode
                        .lookup_entry_mut(cnode_ptr, cnode_depth)?
                        .as_mut()
                        .ok_or(kerr!(ErrKind::SlotIsEmpty))?
                        .as_capability::<CNode>()?;
                    let vspace_slot = todo_root_cnode
                        .lookup_entry_mut(vspace_ptr, vspace_depth)?
                        .as_mut()
                        .ok_or(kerr!(ErrKind::SlotIsEmpty))?
                        .as_capability::<PageTable>()?;
                    tcb_cap.set_cspace(cspace_slot)?;
                    tcb_cap.set_vspace(vspace_slot)?;
                    Ok(())
                }
                InvLabel::TcbWriteReg => {
                    let reg_id = match reg.a3 {
                        0 => 2,  // stack pointer
                        1 => 34, // sepc
                        2 => 10, // a0
                        3 => 11, // a1
                        4 => 12, // a2
                        _ => panic!("cannot set reg {:x}", reg.a2),
                    };
                    let value = reg.a4;
                    tcb_cap.set_registers(&[(reg_id, value)]);
                    Ok(())
                }
                InvLabel::TcbSetIpcBuffer => {
                    let page_ptr = reg.a3;
                    let page_deph = reg.a4 as u32;
                    let page_cap = root_cnode
                        .lookup_entry_mut(page_ptr, page_deph)?
                        .as_mut()
                        .ok_or(kerr!(ErrKind::SlotIsEmpty))?
                        .as_capability::<Page>()?;
                    tcb_cap.set_ipc_buffer(page_cap)
                }
                InvLabel::TcbResume => {
                    tcb_cap.make_runnable();
                    Ok(())
                }
                _ => Err(kerr!(ErrKind::UnknownInvocation)),
            }
        }
        CapabilityType::Notification => {
            // replicate is enough because send or wait operation will never change cap data.
            let mut notify_cap = slot
                .cap_ref_mut()
                .as_capability::<Notification>()?
                .replicate();
            match inv_label {
                InvLabel::NotifySend => {
                    notify_cap.send();
                    Ok(())
                }
                InvLabel::NotifyWait => {
                    if notify_cap.wait(current_tcb) {
                        require_schedule()
                    }
                    Ok(())
                }
                _ => Err(kerr!(ErrKind::UnknownInvocation)),
            }
        }
        CapabilityType::EndPoint => {
            // replicate is enough because send or recv operation will never change cap data.
            let mut ep_cap = slot.cap_ref_mut().as_capability::<Endpoint>()?.replicate();
            match inv_label {
                InvLabel::EpSend => {
                    if ep_cap.send(current_tcb) {
                        require_schedule()
                    }
                    Ok(())
                }
                InvLabel::EpRecv => {
                    if ep_cap.recv(current_tcb) {
                        require_schedule()
                    }
                    Ok(())
                }
                _ => Err(kerr!(ErrKind::UnknownInvocation)),
            }
        }
        CapabilityType::Page => {
            let page_cap = slot.cap_ref_mut().as_capability::<Page>()?;
            match inv_label {
                InvLabel::PageMap => {
                    let page_table_ptr = reg.a3;
                    let page_table_depth = reg.a4 as u32;
                    let vaddr = reg.a5;
                    is_aligned(vaddr, PAGE_SIZE)
                        .then_some(())
                        .ok_or(kerr!(ErrKind::NotAligned, PAGE_SIZE as u16))?;
                    let root_page_table = root_cnode
                        .lookup_entry_mut(page_table_ptr, page_table_depth)?
                        .as_mut()
                        .ok_or(kerr!(ErrKind::SlotIsEmpty))?
                        .cap_ref_mut()
                        .as_capability::<PageTable>()?;
                    let flags = PAGE_U | reg.a6;
                    page_cap.map(root_page_table, vaddr.into(), flags)
                }
                InvLabel::PageUnMap => {
                    todo!()
                }
                _ => Err(kerr!(ErrKind::UnknownInvocation)),
            }
        }
        CapabilityType::PageTable => {
            let page_table_cap = slot.cap_ref_mut().as_capability::<PageTable>()?;
            match inv_label {
                InvLabel::PageTableMap => {
                    let page_table_ptr = reg.a3;
                    let page_table_depth = reg.a4 as u32;
                    let vaddr = reg.a5;
                    is_aligned(vaddr, PAGE_SIZE)
                        .then_some(())
                        .ok_or(kerr!(ErrKind::NotAligned, PAGE_SIZE as u16))?;
                    let root_page_table = root_cnode
                        .lookup_entry_mut(page_table_ptr, page_table_depth)?
                        .as_mut()
                        .ok_or(kerr!(ErrKind::SlotIsEmpty))?
                        .cap_ref_mut()
                        .as_capability::<PageTable>()?;
                    page_table_cap.map(root_page_table, vaddr.into())?;
                    Ok(())
                }
                InvLabel::PageTableUnMap => {
                    todo!()
                }
                _ => Err(kerr!(ErrKind::UnknownInvocation)),
            }
        }
        CapabilityType::Anything => unreachable!(""),
    }
}
