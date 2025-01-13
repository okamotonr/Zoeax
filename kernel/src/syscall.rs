use crate::{
    capability::{cnode::CNodeCap, untyped::UntypedCap, Capability, CapabilityType}, common::{Err, KernelResult}, scheduler::{get_current_tcb, get_current_tcb_mut}, uart::putchar, println,
};


use common::syscall::{CALL, PUTCHAR, UNTYPED_RETYPE, TCB_CONFIGURE};

pub fn handle_syscall(syscall_n: usize) {
    match syscall_n {
        PUTCHAR => {
            let a0 = get_current_tcb().registers.a0;
            putchar(a0 as u8)
        }
        CALL => {
            handle_cap_invocation().unwrap();
        }
        _ => panic!("Unknown system call")
    }
    // increment pc
    get_current_tcb_mut().registers.sepc += 4
}

fn handle_cap_invocation() -> KernelResult<()> {
    let current_tcb = get_current_tcb_mut();
    // change registers with result of invocation.
    let mut root_cnode = CNodeCap::try_from_raw(current_tcb.root_cnode.cap()).unwrap();
    let cap_ptr = current_tcb.registers.a0;
    let inv_label = current_tcb.registers.a1;
    println!("handle cap invocation");
    match inv_label {
        UNTYPED_RETYPE => {
            let dest_cnode_ptr = current_tcb.registers.a2;
            let user_size = current_tcb.registers.a3;
            let num = current_tcb.registers.a4;
            let new_type = CapabilityType::try_from_u8(current_tcb.registers.a5 as u8).unwrap();
            for idx in 0..cap_ptr {
                let entry = root_cnode.lookup_entry(idx);
                println!("{idx:x}, {entry:?}");
            }

            let (src_entry, dest_cnode) = root_cnode.get_src_and_dest(cap_ptr, dest_cnode_ptr, num).unwrap();
            UntypedCap::invoke_retype(
                src_entry, dest_cnode, user_size, num, new_type
            ).unwrap();
            for idx in 0..=cap_ptr+1 {
                let entry = root_cnode.lookup_entry(idx);
                println!("{idx:x}, {entry:?}");
            }
            Ok(())
        }
        TCB_CONFIGURE => {
            todo!()
        }
        _ => {
            Err(Err::UnknownInvocation)
        }
    }
}

