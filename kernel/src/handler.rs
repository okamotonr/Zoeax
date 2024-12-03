use core::{arch::naked_asm, usize};

use crate::{
    println,
    riscv::{r_scause, r_sepc, r_sie, r_stval, w_sepc, w_sie, SIE_STIE},
    uart::putchar,
    process::sleep,
};

use common::{PUTCHAR, SLEEP};

// I wanna use enum;
const USERSOFTWARE: usize = 1;
const SUPERVISOR: usize = 2;
const MACHINESOFTWARE: usize = 3;
const USERTIMER: usize = 4;
const SUPREVISORTIMER: usize = 5;

const INSTMISALIGNED: usize = 0;
const INSTRUCTIONA: usize = 1;
const ECALLUSER: usize = 8;

#[repr(C, packed)]
#[derive(Debug)]
pub struct TrapFrame {
    pub ra: usize,
    pub gp: usize,
    pub tp: usize,
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,
    pub s0: usize,
    pub s1: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
    pub sp: usize,
}

/// 8 and (sd, ld) is valid because of riscv64
#[naked]
pub extern "C" fn trap_entry() {
    unsafe {
        naked_asm!(
        ".balign 8",

        "csrrw sp, sscratch, sp",
        "addi sp, sp, -8 * 31",
        "sd ra,  8 * 0(sp)",
        "sd gp,  8 * 1(sp)",
        "sd tp,  8 * 2(sp)",
        "sd t0,  8 * 3(sp)",
        "sd t1,  8 * 4(sp)",
        "sd t2,  8 * 5(sp)",
        "sd t3,  8 * 6(sp)",
        "sd t4,  8 * 7(sp)",
        "sd t5,  8 * 8(sp)",
        "sd t6,  8 * 9(sp)",
        "sd a0,  8 * 10(sp)",
        "sd a1,  8 * 11(sp)",
        "sd a2,  8 * 12(sp)",
        "sd a3,  8 * 13(sp)",
        "sd a4,  8 * 14(sp)",
        "sd a5,  8 * 15(sp)",
        "sd a6,  8 * 16(sp)",
        "sd a7,  8 * 17(sp)",
        "sd s0,  8 * 18(sp)",
        "sd s1,  8 * 19(sp)",
        "sd s2,  8 * 20(sp)",
        "sd s3,  8 * 21(sp)",
        "sd s4,  8 * 22(sp)",
        "sd s5,  8 * 23(sp)",
        "sd s6,  8 * 24(sp)",
        "sd s7,  8 * 25(sp)",
        "sd s8,  8 * 26(sp)",
        "sd s9,  8 * 27(sp)",
        "sd s10, 8 * 28(sp)",
        "sd s11, 8 * 29(sp)",

        "csrr a0, sscratch",
        "sd a0, 8 * 30(sp)",

        "addi a0, sp, 8*31",
        "csrw sscratch, a0",

        "mv a0, sp",
        "call {handle_trap}",

        "ld ra,  8 * 0(sp)",
        "ld gp,  8 * 1(sp)",
        "ld tp,  8 * 2(sp)",
        "ld t0,  8 * 3(sp)",
        "ld t1,  8 * 4(sp)",
        "ld t2,  8 * 5(sp)",
        "ld t3,  8 * 6(sp)",
        "ld t4,  8 * 7(sp)",
        "ld t5,  8 * 8(sp)",
        "ld t6,  8 * 9(sp)",
        "ld a0,  8 * 10(sp)",
        "ld a1,  8 * 11(sp)",
        "ld a2,  8 * 12(sp)",
        "ld a3,  8 * 13(sp)",
        "ld a4,  8 * 14(sp)",
        "ld a5,  8 * 15(sp)",
        "ld a6,  8 * 16(sp)",
        "ld a7,  8 * 17(sp)",
        "ld s0,  8 * 18(sp)",
        "ld s1,  8 * 19(sp)",
        "ld s2,  8 * 20(sp)",
        "ld s3,  8 * 21(sp)",
        "ld s4,  8 * 22(sp)",
        "ld s5,  8 * 23(sp)",
        "ld s6,  8 * 24(sp)",
        "ld s7,  8 * 25(sp)",
        "ld s8,  8 * 26(sp)",
        "ld s9,  8 * 27(sp)",
        "ld s10, 8 * 28(sp)",
        "ld s11, 8 * 29(sp)",
        "ld sp,  8 * 30(sp)",
        "sret",
        handle_trap = sym handle_trap,
        )
    }
}

fn handle_syscall(trap_frame: &TrapFrame) {
    match trap_frame.a3 {
        PUTCHAR => putchar(trap_frame.a0 as u8),
        SLEEP =>  { sleep(trap_frame.a0) },
        _ => unsafe {
            panic!("Unknown syscall, {:?}", trap_frame);
        },
    }
}

#[no_mangle]
extern "C" fn handle_trap(trap_frame: &TrapFrame) {
    let scause = r_scause();
    let code = scause & !(1 << usize::BITS - 1);
    let stval = r_stval();
    let mut user_pc = r_sepc();

    if (scause >> usize::BITS - 1) == 1 {
        match code {
            SUPREVISORTIMER => {
                // TODO: impl timer intterupt handler
            }
            _ => {
                panic!(
                    "unexpected interrupt scause={:x}, stval={:x}, sepc={:x}",
                    scause, stval, user_pc
                );
            }
        }
    } else {
        match code {
            ECALLUSER => {
                handle_syscall(trap_frame);
                user_pc += 4;
            }
            _ => {
                panic!(
                    "unexpected exception scause={:x}, stval={:x}, sepc={:x}, code={:x}",
                    scause, stval, user_pc, code
                );
            }
        }
    }
    w_sepc(user_pc);
}
