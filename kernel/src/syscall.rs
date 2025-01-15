use crate::{
    capability::{cnode::CNodeCap, notification::NotificationCap, tcb::TCBCap, untyped::UntypedCap, Capability, CapabilityType}, common::{ErrKind, KernelResult}, kerr, object::Registers, scheduler::{get_current_tcb, get_current_tcb_mut, require_schedule}, uart::putchar
};

use common::syscall::{CALL, NOTIFY_WAIT, NOTIFY_SEND, PUTCHAR, SEND, RECV, TCB_CONFIGURE, TCB_RESUME, TCB_WRITE_REG, UNTYPED_RETYPE};

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
    let mut root_cnode = CNodeCap::try_from_raw(current_tcb.root_cnode.cap())?;
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
            let mut tcb_cap = TCBCap::try_from_raw(root_cnode.lookup_entry_mut(cap_ptr)?.cap())?;
            let cspace_slot = root_cnode.lookup_entry_mut(reg.a2)?;
            tcb_cap.set_cspace(cspace_slot)?;
            let vspace = root_cnode.lookup_entry_mut(reg.a3)?;
            tcb_cap.set_vspace(vspace)?;
            Ok(())
        }
        TCB_WRITE_REG => {
            // TODO: currently only support sp and ip, because it is enough to run no arg function.
            // is_stack
            let reg_id = match reg.a2 {
                0 => 2, // stack pointer
                1 => 34, // sepc
                2 => 10, // a0
                _ => panic!("cannot set reg {:x}", reg.a2)
            };
            let value = reg.a3;
            let mut tcb_cap = TCBCap::try_from_raw(root_cnode.lookup_entry_mut(cap_ptr)?.cap())?;
            tcb_cap.set_registers(&[(reg_id, value)]);
            Ok(())
        }
        TCB_RESUME => {
            let mut tcb_cap = TCBCap::try_from_raw(root_cnode.lookup_entry_mut(cap_ptr)?.cap())?;
            tcb_cap.make_runnable();
            Ok(())
        }
        _ => Err(kerr!(ErrKind::UnknownInvocation)),
    }
}

fn handle_send_invocation(reg: &mut Registers) -> KernelResult<()> {
    let current_tcb = get_current_tcb_mut();
    let mut root_cnode = CNodeCap::try_from_raw(current_tcb.root_cnode.cap())?;
    let cap_ptr = reg.a0;
    let inv_label = reg.a1;
    match inv_label {
        NOTIFY_SEND => {
            let mut notify_cap = NotificationCap::try_from_raw(root_cnode.
                lookup_entry_mut(cap_ptr)?.cap())?;
            notify_cap.send();
            Ok(())
        }
        _ => Err(kerr!(ErrKind::UnknownInvocation)),
    }
}

fn handle_recieve_invocation(reg: &mut Registers) -> KernelResult<()> {
    let current_tcb = get_current_tcb_mut();
    let mut root_cnode = CNodeCap::try_from_raw(current_tcb.root_cnode.cap())?;
    let cap_ptr = reg.a0;
    let inv_label = reg.a1;
    match inv_label {
        NOTIFY_WAIT => {
            let mut notify_cap = NotificationCap::try_from_raw(root_cnode.
                lookup_entry_mut(cap_ptr)?.cap())?;
            if notify_cap.wait(current_tcb) {
                require_schedule()
            }
            Ok(())
        }
        _ => Err(kerr!(ErrKind::UnknownInvocation)),
    }
}
