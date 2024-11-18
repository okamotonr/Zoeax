use core::arch::asm;

pub struct SBIRet {
    pub error: u64,
    pub value: u64
}


pub fn sbi_call(arg0: i64, arg1: i64, arg2: i64, arg3: i64, arg4: i64, arg5: i64, fid: i64, eid: i64) -> SBIRet {
    let mut error;
    let mut value;

    unsafe {
        asm!(
            "ecall",
            inout("a0") arg0 => error, inout("a1") arg1 => value,
            in("a2") arg2, in("a3") arg3, in("a4") arg4, in("a5") arg5,
            in("a6") fid, in("a7") eid
        )};

    SBIRet{ error, value }
}

