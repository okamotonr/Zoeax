use crate::{
    capability::{cnode::CNodeCap, tcb::TCBCap, untyped::UntypedCap, Capability, CapabilityType},
    common::{ErrKind, KernelResult},
    kerr, println,
    scheduler::{get_current_tcb, get_current_tcb_mut},
    uart::putchar,
};

use common::syscall::{CALL, PUTCHAR, TCB_CONFIGURE, TCB_RESUME, TCB_WRITE_REG, UNTYPED_RETYPE};

pub fn handle_syscall(syscall_n: usize) {
    match syscall_n {
        PUTCHAR => {
            let a0 = get_current_tcb().registers.a0;
            putchar(a0 as u8)
        }
        CALL => {
            handle_cap_invocation().unwrap();
        }
        _ => panic!("Unknown system call"),
    }
    // increment pc
    get_current_tcb_mut().registers.sepc += 4
}

fn handle_cap_invocation() -> KernelResult<()> {
    let current_tcb = get_current_tcb_mut();
    // change registers with result of invocation.
    let mut root_cnode = CNodeCap::try_from_raw(current_tcb.root_cnode.cap())?;
    let cap_ptr = current_tcb.registers.a0;
    let inv_label = current_tcb.registers.a1;
    println!("handle cap invocation");
    match inv_label {
        UNTYPED_RETYPE => {
            let dest_cnode_ptr = current_tcb.registers.a2;
            let user_size = current_tcb.registers.a3;
            let num = current_tcb.registers.a4;
            let new_type = CapabilityType::try_from_u8(current_tcb.registers.a5 as u8)?;
            let (src_entry, dest_cnode) =
                root_cnode.get_src_and_dest(cap_ptr, dest_cnode_ptr, num)?;
            UntypedCap::invoke_retype(src_entry, dest_cnode, user_size, num, new_type)?;
            Ok(())
        }
        TCB_CONFIGURE => {
            // TODO: lookup entry first to be able to rollback
            // TODO: we have to do something to make rust ownership be calm down.
            let mut tcb_cap = TCBCap::try_from_raw(root_cnode.lookup_entry_mut(cap_ptr)?.cap())?;
            let cspace_slot = root_cnode.lookup_entry_mut(current_tcb.registers.a2)?;
            tcb_cap.set_cspace(cspace_slot)?;
            let vspace = root_cnode.lookup_entry_mut(current_tcb.registers.a3)?;
            tcb_cap.set_vspace(vspace)?;
            println!("{:?}", tcb_cap.get_tcb());
            Ok(())
        }
        TCB_WRITE_REG => {
            // TODO: currently only support sp and ip, because it is enough to run no arg function.
            // is_stack
            let reg_id = if current_tcb.registers.a2 == 0 {
                2 // stack pointer
            } else {
                34 // sepc
            };
            let value = current_tcb.registers.a3;
            let mut tcb_cap = TCBCap::try_from_raw(root_cnode.lookup_entry_mut(cap_ptr)?.cap())?;
            tcb_cap.set_registers(&[(reg_id, value)]);
            println!("{:?}", tcb_cap.get_tcb());
            Ok(())
        }
        TCB_RESUME => {
            let mut tcb_cap = TCBCap::try_from_raw(root_cnode.lookup_entry_mut(cap_ptr)?.cap())?;
            tcb_cap.make_runnable();
            println!("{:?}", tcb_cap.get_tcb());
            Ok(())
        }
        _ => Err(kerr!(ErrKind::UnknownInvocation)),
    }
}
