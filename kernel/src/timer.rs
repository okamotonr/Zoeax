use crate::riscv::r_time;
use crate::sbi::{sbi_call, SBIRet};

pub const MTIME_PER_1MS: usize = 10000;

const TIMER: i64 = 0x54494D45;

pub fn set_timer(val: usize) -> SBIRet {
    let rd_time = r_time();
    sbi_call((val + rd_time) as i64, 0, 0, 0, 0, 0, 0, TIMER)
}
