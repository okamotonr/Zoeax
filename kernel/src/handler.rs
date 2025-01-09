use core::{arch::naked_asm, usize};

use crate::{
    riscv::{r_scause, r_sepc, r_stval},
    timer::set_timer,
    syscall::handle_syscall,
};


// I wanna use enum;
/// Interrupts, 
const SUPERVISORSOFTWARE: usize = 1;
const SUPREVISORTIMER: usize = 5;
const SUPREVISOREXTERNAL: usize = 9;
const COUNTER_OVERFLOW: usize = 13;

/// Exceptions
const IMISSALIGNED: usize = 0;
const IACCESSFAULT: usize = 1;
const ILEAGALI: usize = 2;
const BREAKPOINT: usize = 3;
const LMISSALIGNED: usize = 4;
const LACCESSFAULT: usize = 5;
const SAMISSALIGNED: usize = 6;
const SAACCESSFAULT: usize = 7;
const ECALLUSER: usize = 8;
const ECALLSUPERVIOSR: usize = 9;
const IPAGEFAULT: usize = 12;
const LPAGEFAULT: usize = 13;
const SAPAGEFAULT: usize = 15;

#[repr(C, packed)]
#[derive(Debug)]
pub struct TrapFrame {
    pub pc: usize,
    pub sstatus: usize,
    pub sp: usize,
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
}

/// 8 and (sd, ld) is valid because of riscv64
/// This is trap handler entry fnction.
/// Save context when trap was occured (trap frame),
/// then call trap_handler
/// after that, restore context and call sret.
#[naked]
pub extern "C" fn trap_entry() {
    unsafe {
        naked_asm!(
        ".balign 8",

        // sscratch has cpu var address
        // tmp = tp
        // tp = &CPU_VAR
        // sscratch = tmp
        "csrrw tp, sscratch, tp",

        // CPU_VAR.sscratch = a0
        "sd a0, 0(tp)",

        // whether trap occured in kernel mode or.
        "csrr a0, sstatus",
        "andi a0, a0, (1 << 8)",
        "bnez a0, 1f",

        // load kernel stack pointer to a0
        "ld a0, 8 * 1(tp)",
        "j 2f",


        "1:",
        // already kernel mode so use same sp as before.
        "mv a0, sp",

        "2:",

        // a0 has stack pointer which will be used in trap handler.
        "addi a0, a0, -8 * 33",
        "sd ra,  8 * 2(a0)",
        "sd sp,  8 * 3(a0)",
        "sd gp,  8 * 4(a0)",
        "sd tp,  8 * 5(a0)",
        "sd t0,  8 * 6(a0)",
        "sd t1,  8 * 7(a0)",
        "sd t2,  8 * 8(a0)",
        "sd t3,  8 * 9(a0)",
        "sd t4,  8 * 10(a0)",
        "sd t5,  8 * 11(a0)",
        "sd t6,  8 * 12(a0)",
        // "sd a0,  8 * 13(sp)",
        "sd a1,  8 * 14(a0)",
        "sd a2,  8 * 15(a0)",
        "sd a3,  8 * 16(a0)",
        "sd a4,  8 * 17(a0)",
        "sd a5,  8 * 18(a0)",
        "sd a6,  8 * 19(a0)",
        "sd a7,  8 * 20(a0)",
        "sd s0,  8 * 21(a0)",
        "sd s1,  8 * 22(a0)",
        "sd s2,  8 * 23(a0)",
        "sd s3,  8 * 24(a0)",
        "sd s4,  8 * 25(a0)",
        "sd s5,  8 * 26(a0)",
        "sd s6,  8 * 27(a0)",
        "sd s7,  8 * 28(a0)",
        "sd s8,  8 * 29(a0)",
        "sd s9,  8 * 30(a0)",
        "sd s10, 8 * 31(a0)",
        "sd s11, 8 * 32(a0)",

        "mv sp, a0",

        // a0 = sscratch(= tp)
        // sscratch = tp(= &CPU_VAR)
        "csrrw a0, sscratch, tp",
        "sd a0,  8 * 5(sp)",

        // restore a0
        "ld a0, (tp)",
        "sd a0, 8 *13(sp)",

        "csrr a0, sepc",
        "sd a0, 8 * 0(sp)",
        "csrr a0, sstatus",
        "sd a0, 8 * 1(sp)",

        "mv a0, sp",
        "call {handle_trap}",

        "ld a0, 8 * 0(sp)",
        "csrw sepc, a0",
        "ld a0, 8 * 1(sp)",
        "csrw sstatus, a0",

        "ld ra,  8 * 2(sp)",
        "ld gp,  8 * 4(sp)",
        "ld tp,  8 * 5(sp)",
        "ld t0,  8 * 6(sp)",
        "ld t1,  8 * 7(sp)",
        "ld t2,  8 * 8(sp)",
        "ld t3,  8 * 9(sp)",
        "ld t4,  8 * 10(sp)",
        "ld t5,  8 * 11(sp)",
        "ld t6,  8 * 12(sp)",
        "ld a0,  8 * 13(sp)",
        "ld a1,  8 * 14(sp)",
        "ld a2,  8 * 15(sp)",
        "ld a3,  8 * 16(sp)",
        "ld a4,  8 * 17(sp)",
        "ld a5,  8 * 18(sp)",
        "ld a6,  8 * 19(sp)",
        "ld a7,  8 * 20(sp)",
        "ld s0,  8 * 21(sp)",
        "ld s1,  8 * 22(sp)",
        "ld s2,  8 * 23(sp)",
        "ld s3,  8 * 24(sp)",
        "ld s4,  8 * 25(sp)",
        "ld s5,  8 * 26(sp)",
        "ld s6,  8 * 27(sp)",
        "ld s7,  8 * 28(sp)",
        "ld s8,  8 * 29(sp)",
        "ld s9,  8 * 30(sp)",
        "ld s10, 8 * 31(sp)",
        "ld s11, 8 * 32(sp)",
        "ld sp,  8 * 3(sp)",

        "sret",
        handle_trap = sym handle_trap,
        )
    }
}

// TODO: when adapt multi hart, make sscratch specify same hart cpu var when trap happened in kernel mode.

#[no_mangle]
extern "C" fn handle_trap(trap_frame: &mut TrapFrame) {
    let scause = r_scause();
    let code = scause & !(1 << usize::BITS - 1);
    let stval = r_stval();
    let user_pc = r_sepc();

    if (scause >> usize::BITS - 1) == 1 {
    //  interrupt
        match code {
            SUPREVISORTIMER => {
                set_timer(10000);
            }
            SUPERVISORSOFTWARE => {
                panic!(
                    "supervisor software scause={:x}, stval={:x}, sepc={:x}",
                    scause, stval, user_pc
                )
            },
            SUPREVISOREXTERNAL => {
                panic!(
                    "supervisor external scause={:x}, stval={:x}, sepc={:x}",
                    code, stval, user_pc
                )
            },
            COUNTER_OVERFLOW => {
                panic!(
                    "counter overflow scause={:x}, stval={:x}, sepc={:x}",
                    code, stval, user_pc
                )
            },
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
                handle_syscall(trap_frame.a0, trap_frame.a1, trap_frame.a2, trap_frame.a3, trap_frame.a4);
                // increment pc
                trap_frame.pc += 4;
            }
            IMISSALIGNED => {
                panic!(
                    "inst missaligned scause={:x}, stval={:x}, sepc={:x}",
                    scause, stval, user_pc);
            }
            IACCESSFAULT => {
                panic!(
                    "inst access fault scause={:x}, stval={:x}, sepc={:x}",
                    scause, stval, user_pc);
            }
            ILEAGALI => {
                panic!(
                    "inst ileagal scause={:x}, stval={:x}, sepc={:x}",
                    scause, stval, user_pc);
            }
            BREAKPOINT => {
                panic!(
                    "break point scause={:x}, stval={:x}, sepc={:x}",
                    scause, stval, user_pc);
            }
            LMISSALIGNED => {
                panic!(
                    "load missaligned scause={:x}, stval={:x}, sepc={:x}",
                    scause, stval, user_pc);
            }
            LACCESSFAULT => {
                panic!(
                    "load access fault scause={:x}, stval={:x}, sepc={:x}",
                    scause, stval, user_pc);
            }
            SAMISSALIGNED => {
                panic!(
                    "store/amo missaligned scause={:x}, stval={:x}, sepc={:x}",
                    scause, stval, user_pc);
            }
            SAACCESSFAULT => {
                panic!(
                    "store/amo access fault scause={:x}, stval={:x}, sepc={:x}",
                    scause, stval, user_pc);
            }
            ECALLSUPERVIOSR => {
                panic!(
                    "ecall from supervisor scause={:x}, stval={:x}, sepc={:x}",
                    scause, stval, user_pc);
            }
            IPAGEFAULT => {
                panic!(
                    "inst page fault scause={:x}, stval={:x}, sepc={:x}",
                    scause, stval, user_pc);
            }
            LPAGEFAULT => {
                panic!(
                    "load page fault scause={:x}, stval={:x}, sepc={:x}",
                    scause, stval, user_pc);
            }
            SAPAGEFAULT => {
                panic!(
                    "store/amo page fault, scause={:x}, stval={:x}, sepc={:x}",
                    scause, stval, user_pc);
            }
            _ => {
                panic!(
                    "unexpected exception scause={:x}, stval={:x}, sepc={:x}, code={:x}",
                    scause, stval, user_pc, code
                );
            }
        }
    }
}
