use crate::{
    address::PAGE_SIZE, capability::{
        cnode::CNodeCap, endpoint::EndPointCap, notification::NotificationCap, page_table::{PageCap, PageTableCap}, tcb::TCBCap, untyped::UntypedCap, Capability, CapabilityType
    }, common::{is_aligned, ErrKind, KernelResult}, kerr, object::{page_table::PAGE_U, CNodeEntry, Registers}, println, scheduler::{get_current_tcb_mut, require_schedule}, uart::putchar
};

use common::syscall::{
    CALL, CNODE_COPY, CNODE_MINT, CNODE_MOVE, NOTIFY_SEND, NOTIFY_WAIT, PAGE_MAP, PAGE_TABLE_MAP, PUTCHAR, RECV, SEND, TCB_CONFIGURE, TCB_RESUME, TCB_SET_IPC_BUFFER, TCB_WRITE_REG, UNTYPED_RETYPE, EP_SEND, EP_RECV
};

pub fn handle_syscall(syscall_n: usize, reg: &mut Registers) {
    let syscall_ret = match syscall_n {
        PUTCHAR => {
            let a0 = reg.a0;
            putchar(a0 as u8);
            Ok(())
        }
        CALL => {
            handle_call_invocation(reg)
        }
        SEND => {
            handle_send_invocation(reg)
        }
        RECV => {
            handle_recieve_invocation(reg)
        }
        _ => panic!("Unknown system call"),
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

fn handle_call_invocation(reg: &mut Registers) -> KernelResult<()> {
    let current_tcb = get_current_tcb_mut();
    let mut root_cnode = CNodeCap::try_from_raw(current_tcb.root_cnode.as_mut().unwrap().cap())?;
    let cap_ptr = reg.a0;
    let inv_label = reg.a1;
    match inv_label {
        UNTYPED_RETYPE => {
            let dest_cnode_ptr = reg.a2;
            let user_size = reg.a3;
            let num = reg.a4;
            let new_type = CapabilityType::try_from_u8(reg.a5 as u8)?;
            let (src_entry, dest_cnode) =
                root_cnode.get_src_and_dest(cap_ptr, dest_cnode_ptr, num)?;
            UntypedCap::invoke_retype(src_entry, dest_cnode, user_size, num, new_type)?;
            Ok(())
        }
        TCB_CONFIGURE => {
            // TODO: lookup entry first to be able to rollback
            // TODO: we have to do something to make rust ownership be calm down.
            let mut tcb_cap = TCBCap::try_from_raw(
                root_cnode
                    .lookup_entry_mut_one_level(cap_ptr)?
                    .as_mut()
                    .unwrap()
                    .cap(),
            )?;
            let cspace_slot = root_cnode.lookup_entry_mut_one_level(reg.a2)?;
            //let vspace = root_cnode.lookup_entry_mut_one_level(reg.a3)?;
            tcb_cap.set_cspace(cspace_slot.as_mut().unwrap())?;
            let vspace = root_cnode.lookup_entry_mut_one_level(reg.a3)?;
            tcb_cap.set_vspace(vspace.as_mut().unwrap())?;
            Ok(())
        }
        TCB_WRITE_REG => {
            // TODO: currently only support sp, ip, and a0.
            // is_stack
            let reg_id = match reg.a2 {
                0 => 2,  // stack pointer
                1 => 34, // sepc
                2 => 10, // a0
                3 => 11, // a1
                4 => 12, // a2
                _ => panic!("cannot set reg {:x}", reg.a2),
            };
            let value = reg.a3;
            let mut tcb_cap = TCBCap::try_from_raw(
                root_cnode
                    .lookup_entry_mut_one_level(cap_ptr)?
                    .as_mut()
                    .unwrap()
                    .cap(),
            )?;
            tcb_cap.set_registers(&[(reg_id, value)]);
            Ok(())
        }
        TCB_SET_IPC_BUFFER => {
            let page_ptr = reg.a2;
            let mut tcb_cap = TCBCap::try_from_raw(
                root_cnode
                    .lookup_entry_mut_one_level(cap_ptr)?
                    .as_mut()
                    .unwrap()
                    .cap(),
            )?;
            let page_cap = root_cnode.lookup_entry_mut_one_level(page_ptr)?
                    .as_mut()
                    .unwrap();
            // TODO: validate this page is mapped in this tcb's address space.
            tcb_cap.set_ipc_buffer(page_cap)?;
            Ok(())
        }
    
        TCB_RESUME => {
            let mut tcb_cap = TCBCap::try_from_raw(
                root_cnode
                    .lookup_entry_mut_one_level(cap_ptr)?
                    .as_mut()
                    .unwrap()
                    .cap(),
            )?;
            tcb_cap.make_runnable();
            Ok(())
        },
        PAGE_MAP | PAGE_TABLE_MAP => {
            let page_table_ptr = reg.a2;
            let vaddr = reg.a3;
            is_aligned(vaddr, PAGE_SIZE).then_some(()).ok_or(kerr!(ErrKind::NotAligned, PAGE_SIZE as u16))?;
            let mut page_table_cap = PageTableCap::try_from_raw(
                root_cnode
                    .lookup_entry_mut_one_level(page_table_ptr)?
                    .as_mut()
                    .unwrap()
                    .cap(),
            )?;
            if inv_label == PAGE_MAP {
                // TODO: interpret flags
                let flags = PAGE_U | reg.a4;
                let mut page = PageCap::try_from_raw(
                    root_cnode
                        .lookup_entry_mut_one_level(cap_ptr)?
                        .as_mut()
                        .unwrap()
                        .cap(),
                )?;
                page.map(&mut page_table_cap, vaddr.into(), flags)?;
                let page_entry = root_cnode.lookup_entry_mut_one_level(cap_ptr)?.as_mut().ok_or(kerr!(ErrKind::CapNotFound))?;
                page_entry.set_cap(page.get_raw_cap());
            } else {
                let mut page = PageTableCap::try_from_raw(
                    root_cnode
                        .lookup_entry_mut_one_level(cap_ptr)?
                        .as_mut()
                        .unwrap()
                        .cap(),
                )?;
                page.map(&mut page_table_cap, vaddr.into())?;
                let page_entry = root_cnode.lookup_entry_mut_one_level(cap_ptr)?.as_mut().ok_or(kerr!(ErrKind::CapNotFound))?;
                page_entry.set_cap(page.get_raw_cap());
            }
            Ok(())
        }
        CNODE_COPY | CNODE_MINT | CNODE_MOVE => {
            let src_depth = (reg.a3 >> 31) as u32;
            let dest_depth = reg.a3 as u32;
            let mut src_root = CNodeCap::try_from_raw(
                root_cnode
                    .lookup_entry_mut_one_level(cap_ptr)?
                    .as_mut()
                    .unwrap()
                    .cap(),
            )?;
            let src_slot = src_root.lookup_entry_mut(reg.a2, src_depth)?;
            let src_entry = src_slot.as_mut().ok_or(kerr!(ErrKind::SlotIsEmpty))?;

            let mut dest_root = CNodeCap::try_from_raw(
                root_cnode
                    .lookup_entry_mut_one_level(reg.a4)?
                    .as_mut()
                    .unwrap()
                    .cap(),
            )?;
            let dest_slot = dest_root.lookup_entry_mut(reg.a5, dest_depth)?;
            if dest_slot.is_some() {
                Err(kerr!(ErrKind::NotEmptySlot))
            } else {
                let raw_cap = src_entry.cap();
                // TODO: Whether this cap is derivable
                let mut cap = raw_cap;
                if inv_label == CNODE_MINT {
                    let cap_val = reg.a6;
                    cap.set_cap_dep_val(cap_val);
                }

                let mut new_slot = CNodeEntry::new_with_rawcap(cap);
                if inv_label == CNODE_MOVE {
                    new_slot.replace(src_entry);
                    *src_slot = None
                } else {
                    new_slot.insert(src_entry);
                }
                *dest_slot = Some(new_slot);
                Ok(())
            }
        },
        _ => Err(kerr!(ErrKind::UnknownInvocation)),
    }
}

fn handle_send_invocation(reg: &mut Registers) -> KernelResult<()> {
    let current_tcb = get_current_tcb_mut();
    let mut root_cnode = CNodeCap::try_from_raw(current_tcb.root_cnode.as_mut().unwrap().cap())?;
    let cap_ptr = reg.a0;
    let inv_label = reg.a1;
    match inv_label {
        NOTIFY_SEND => {
            let mut notify_cap = NotificationCap::try_from_raw(
                root_cnode
                    .lookup_entry_mut_one_level(cap_ptr)?
                    .as_mut()
                    .unwrap()
                    .cap(),
            )?;
            notify_cap.send();
            Ok(())
        }
        EP_SEND => {
            let mut ep_cap = EndPointCap::try_from_raw(
                root_cnode
                    .lookup_entry_mut_one_level(cap_ptr)?
                    .as_mut()
                    .unwrap()
                    .cap(),
            )?;
            if ep_cap.send(current_tcb) {
                require_schedule()
            }
            Ok(())

        }
        _ => Err(kerr!(ErrKind::UnknownInvocation)),
    }
}

fn handle_recieve_invocation(reg: &mut Registers) -> KernelResult<()> {
    let current_tcb = get_current_tcb_mut();
    let mut root_cnode = CNodeCap::try_from_raw(current_tcb.root_cnode.as_mut().unwrap().cap())?;
    let cap_ptr = reg.a0;
    let inv_label = reg.a1;
    match inv_label {
        NOTIFY_WAIT => {
            let mut notify_cap = NotificationCap::try_from_raw(
                root_cnode
                    .lookup_entry_mut_one_level(cap_ptr)?
                    .as_mut()
                    .unwrap()
                    .cap(),
            )?;
            if notify_cap.wait(current_tcb) {
                require_schedule()
            }
            Ok(())
        }
        EP_RECV => {
            let mut ep_cap = EndPointCap::try_from_raw(
                root_cnode
                    .lookup_entry_mut_one_level(cap_ptr)?
                    .as_mut()
                    .unwrap()
                    .cap(),
            )?;
            if ep_cap.recv(current_tcb) {
                require_schedule()
            }
            Ok(())
        }
        _ => Err(kerr!(ErrKind::UnknownInvocation)),
    }
}
