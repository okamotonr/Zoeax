use crate::{
    capability::{
        cnode::CNodeCap, notification::NotificationCap, tcb::TCBCap, untyped::UntypedCap,
        Capability, CapabilityType,
    },
    common::{ErrKind, KernelResult},
    kerr,
    object::{CNodeEntry, Registers},
    scheduler::{get_current_tcb_mut, require_schedule},
    uart::putchar,
};

use common::syscall::{
    CALL, CNODE_COPY, CNODE_MINT, NOTIFY_SEND, NOTIFY_WAIT, PUTCHAR, RECV, SEND, TCB_CONFIGURE, TCB_RESUME, TCB_WRITE_REG, UNTYPED_RETYPE
};

pub fn handle_syscall(syscall_n: usize, reg: &mut Registers) {
    reg.sepc += 4;
    match syscall_n {
        PUTCHAR => {
            let a0 = reg.a0;
            putchar(a0 as u8);
        }
        CALL => {
            handle_call_invocation(reg).unwrap();
        }
        SEND => {
            handle_send_invocation(reg).unwrap();
        }
        RECV => {
            handle_recieve_invocation(reg).unwrap();
        }
        _ => panic!("Unknown system call"),
    }
    // increment pc
}

fn handle_call_invocation(reg: &mut Registers) -> KernelResult<()> {
    let current_tcb = get_current_tcb_mut();
    // change registers with result of invocation.
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
            // TODO: currently only support sp and ip, because it is enough to run no arg function.
            // is_stack
            let reg_id = match reg.a2 {
                0 => 2,  // stack pointer
                1 => 34, // sepc
                2 => 10, // a0
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
        CNODE_COPY | CNODE_MINT => {
            let src_depth = (reg.a3 >> 31) as u32;
            let dest_depth = reg.a3 as u32;
            let mut src_root = CNodeCap::try_from_raw(root_cnode.lookup_entry_mut_one_level(cap_ptr)?.as_mut().unwrap().cap())?;
            let src_slot = src_root.lookup_entry_mut(reg.a2, src_depth)?.as_mut().ok_or(kerr!(ErrKind::SlotIsEmpty))?;

            let mut dest_root = CNodeCap::try_from_raw(root_cnode.lookup_entry_mut_one_level(reg.a4)?.as_mut().unwrap().cap())?;
            let dest_slot = dest_root.lookup_entry_mut(reg.a5, dest_depth)?;
            if dest_slot.is_some() {
                Err(kerr!(ErrKind::NotEmptySlot))
            } else {
                let raw_cap = src_slot.cap();
                // TODO: Whether this cap is derivable 
                let mut cap = raw_cap;
                if inv_label == CNODE_MINT {
                    let cap_val = reg.a6;
                    cap.set_cap_dep_val(cap_val);
                }
                let mut new_slot = CNodeEntry::new_with_rawcap(cap);
                new_slot.insert(src_slot);
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
        _ => Err(kerr!(ErrKind::UnknownInvocation)),
    }
}
