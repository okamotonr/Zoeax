use core::arch::asm;

/// supervisor-level software interrupts
pub const SIE_SSIE: usize = 1 << 1;  
/// supervisor-level timer interrupts
pub const SIE_STIE: usize = 1 << 5;
/// supervisor-level external interrupts
pub const SIE_SEIE: usize = 1 << 9;

pub const SIP_STIP: usize = 1 << 5;

/// interrupt-enable bit
pub const SSTATUS_SIE: usize = 1 << 1;
/// interrupt-enable bit active prior to the trap
pub const SSTATUS_SPIE: usize = 1 << 5;
/// previous privilege mode
pub const SSTATUS_SPP: usize = 1 << 8;
/// permit supervisor user memory access
pub const SSTATUS_SUM: usize = 1 << 18;

macro_rules! read_csr {
    ($csr:expr) => {
        {
            let mut ret;
            unsafe {
                asm!(concat!("csrr ",  "{r}, ", $csr), r = out(reg) ret);
            };
            ret
        }
    }
}

macro_rules! write_csr {
    ($csr:expr, $value:expr) => {
        unsafe {
            asm!(concat!("csrw ", $csr, ", {r}"), r = in(reg) $value);
        }
    };
}


#[inline]
pub fn r_scause() -> usize {
    read_csr!("scause")
}

#[inline]
pub fn r_stval() -> usize {
    read_csr!("stval")
}


#[inline]
pub fn r_sepc() -> usize {
    read_csr!("sepc")
}

#[inline]
pub fn r_sie() -> usize {
    read_csr!("sie")
}

#[inline]
pub fn r_sstatus() -> usize {
    read_csr!("sstatus")
}

#[inline]
pub fn r_sip() -> usize {
    read_csr!("sip")
}

#[inline]
pub fn w_sie(val: usize) {
    write_csr!("sie", val)
}

#[inline]
pub fn w_sip(val: usize) {
    write_csr!("sip", val)
}

#[inline]
pub fn w_sepc(addr: usize) {
    write_csr!("sepc", addr)
}

#[inline]
pub fn w_sstatus(val: usize) {
    write_csr!("sstatus", val)
}

#[inline]
pub fn w_stvec(addr: usize) {
    write_csr!("stvec", addr)
}

#[inline]
pub fn w_sscratch(val: usize) {
    write_csr!("sscratch", val)
}

#[inline]
pub fn wfi() {
    unsafe {
        asm!("wfi", options(nomem, nostack));
    }
}

#[inline]
pub fn r_time() -> usize {
    let mut ret;
    unsafe {
        asm!("rdtime {}", out(reg) ret);
    };
    ret
}

#[inline]
pub fn r_satp() -> usize {
    read_csr!("satp")
}


