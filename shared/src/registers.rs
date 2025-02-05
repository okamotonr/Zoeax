use core::ops::{Index, IndexMut};
#[derive(Debug, Clone, Copy)]
pub enum Register {
    Ra,
    Sp,
    Gp,
    Tp,
    T0,
    T1,
    T2,
    S0,
    S1,
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    S8,
    S9,
    S10,
    S11,
    T3,
    T4,
    T5,
    T6,

    SCause,
    SStatus,
    SEpc,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Registers {
    pub ra: usize,
    pub sp: usize,
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

    // End of general purpose registers
    pub scause: usize,
    pub sstatus: usize,
    pub sepc: usize,
}

impl Registers {
    pub const fn null() -> Self {
        Self {
            ra: 0,
            sp: 0,
            gp: 0,
            tp: 0,
            t0: 0,
            t1: 0,
            t2: 0,
            t3: 0,
            t4: 0,
            t5: 0,
            t6: 0,
            a0: 0,
            a1: 0,
            a2: 0,
            a3: 0,
            a4: 0,
            a5: 0,
            a6: 0,
            a7: 0,
            s0: 0,
            s1: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            s9: 0,
            s10: 0,
            s11: 0,
            scause: 0,
            sstatus: 0,
            sepc: 0,
        }
    }
}

impl Index<Register> for Registers {
    type Output = usize;

    fn index(&self, reg: Register) -> &Self::Output {
        match reg {
            Register::Ra => &self.ra,
            Register::Sp => &self.sp,
            Register::Gp => &self.gp,
            Register::Tp => &self.tp,
            Register::T0 => &self.t0,
            Register::T1 => &self.t1,
            Register::T2 => &self.t2,
            Register::T3 => &self.t3,
            Register::T4 => &self.t4,
            Register::T5 => &self.t5,
            Register::T6 => &self.t6,
            Register::A0 => &self.a0,
            Register::A1 => &self.a1,
            Register::A2 => &self.a2,
            Register::A3 => &self.a3,
            Register::A4 => &self.a4,
            Register::A5 => &self.a5,
            Register::A6 => &self.a6,
            Register::A7 => &self.a7,
            Register::S0 => &self.s0,
            Register::S1 => &self.s1,
            Register::S2 => &self.s2,
            Register::S3 => &self.s3,
            Register::S4 => &self.s4,
            Register::S5 => &self.s5,
            Register::S6 => &self.s6,
            Register::S7 => &self.s7,
            Register::S8 => &self.s8,
            Register::S9 => &self.s9,
            Register::S10 => &self.s10,
            Register::S11 => &self.s11,
            Register::SCause => &self.scause,
            Register::SStatus => &self.sstatus,
            Register::SEpc => &self.sepc,
        }
    }
}

impl IndexMut<Register> for Registers {
    fn index_mut(&mut self, reg: Register) -> &mut Self::Output {
        match reg {
            Register::Ra => &mut self.ra,
            Register::Sp => &mut self.sp,
            Register::Gp => &mut self.gp,
            Register::Tp => &mut self.tp,
            Register::T0 => &mut self.t0,
            Register::T1 => &mut self.t1,
            Register::T2 => &mut self.t2,
            Register::T3 => &mut self.t3,
            Register::T4 => &mut self.t4,
            Register::T5 => &mut self.t5,
            Register::T6 => &mut self.t6,
            Register::A0 => &mut self.a0,
            Register::A1 => &mut self.a1,
            Register::A2 => &mut self.a2,
            Register::A3 => &mut self.a3,
            Register::A4 => &mut self.a4,
            Register::A5 => &mut self.a5,
            Register::A6 => &mut self.a6,
            Register::A7 => &mut self.a7,
            Register::S0 => &mut self.s0,
            Register::S1 => &mut self.s1,
            Register::S2 => &mut self.s2,
            Register::S3 => &mut self.s3,
            Register::S4 => &mut self.s4,
            Register::S5 => &mut self.s5,
            Register::S6 => &mut self.s6,
            Register::S7 => &mut self.s7,
            Register::S8 => &mut self.s8,
            Register::S9 => &mut self.s9,
            Register::S10 => &mut self.s10,
            Register::S11 => &mut self.s11,
            Register::SCause => &mut self.scause,
            Register::SStatus => &mut self.sstatus,
            Register::SEpc => &mut self.sepc,
        }
    }
}
