#[repr(u8)]
pub enum SysCallNo {
    Print = 0,
    Call = 1,
    Send = 2,
    Recv = 3,
}

