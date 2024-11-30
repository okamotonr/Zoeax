use core::arch::asm;

#[inline]
pub fn r_scause() -> usize {
    let mut ret;
    unsafe {
        asm!("csrr {}, scause", out(reg) ret);
    };
    ret
}

#[inline]
pub fn r_stval() -> usize {
    let mut ret;
    unsafe {
        asm!("csrr {}, stval", out(reg) ret);
    };
    ret
}

#[inline]
pub fn r_sepc() -> usize {
    let mut ret;
    unsafe {
        asm!("csrr {}, sepc", out(reg) ret);
    };
    ret
}

#[inline]
pub fn w_stvec(addr: usize) {
    unsafe {
        asm!("csrw stvec, {0}", in(reg) addr);
    }
}

#[inline]
pub fn w_sepc(addr: usize) {
    unsafe {
        asm!("csrw sepc, {0}", in(reg) addr);
    }
}

#[inline]
pub fn wfi() {
    unsafe {
        asm!("wfi", options(nomem, nostack));
    }
}

#[inline]
pub fn r_mhartid() -> usize {
    let mut ret;
    unsafe {
        asm!("csrr {}, mhartid", out(reg) ret);
    };
    ret
}
