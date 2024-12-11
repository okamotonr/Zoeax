use crate::{
    common::{Err, KernelResult},
    memory::{copy_from_user, copy_to_user, VirtAddr},
    println,
    process::{ProcessStatus, Process},
    scheduler::{find_proc_by_id, yield_proc, CURRENT_PROC},
    uart::putchar,
    scheduler::sleep,
};

use core::ptr;

use common::syscall::{Message, PUTCHAR, RECV, SEND, SLEEP};

pub fn handle_syscall(a0: usize, a1: usize, _a2: usize, _a3: usize, syscall_n: usize) {
    match syscall_n {
        PUTCHAR => putchar(a0 as u8),
        SLEEP => {
            sleep(a0);
        }
        SEND => {
            println!("send called");
            println!("arg0 {}, arg1 {:x}", a0, a1);
            sys_send(a0, a1.into()).unwrap()
        }
        RECV => {
            sys_recv(a0).unwrap();
        }
        _ => {
            panic!("Unknown syscall, {:?}", syscall_n);
        }
    }
}

fn sys_send(pid: usize, mptr: VirtAddr) -> KernelResult<()> {
    unsafe {
        if (*CURRENT_PROC).pid == pid {
            println!("cannot send message to current");
        }
    }
    let target_proc = find_proc_by_id(pid).ok_or(Err::ProcessNotFound)?;
    if target_proc.is_unused() {
        println!(
            "{:?}, {:?}, {:?}",
            target_proc.pid, target_proc.status, target_proc.stack_top
        );
        return Err(Err::ProcessNotFound);
    }
    if !target_proc.is_waiting() {
        unsafe {
            target_proc.waiter = &mut *(*CURRENT_PROC) as *mut Process;
            (*CURRENT_PROC).waiting();
            yield_proc()
        }
    }

    // woken up by target process
    let mut send_m = Message::new();
    unsafe { copy_from_user(mptr, &mut send_m)? };
    target_proc.set_message(send_m)?;
    Ok(())
}

fn sys_recv(mptr: usize) -> KernelResult<()> {
    unsafe {
        if !(*CURRENT_PROC).waiter.is_null() {
            (*(*CURRENT_PROC).waiter).status = ProcessStatus::Runnable;
            (*CURRENT_PROC).waiter = ptr::null_mut();
        }
        println!("{:?} i am waiting", (*CURRENT_PROC).pid);
        (*CURRENT_PROC).waiting();
        yield_proc()
    }
    unsafe {
        let src = &(*CURRENT_PROC).message.unwrap() as *const Message as usize;
        copy_to_user::<Message>(src.into(), mptr.into())?;
        (*CURRENT_PROC).message = None;
    }
    unsafe {
    println!("rcv, {:?}", (*CURRENT_PROC).pid);
    }
    Ok(())
}
