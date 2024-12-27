pub fn is_aligned(value: usize, align: usize) -> bool {
    value % align == 0
}

/// align should be power of 2.
pub fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Err {
    NoMemory,
    TooManyTasks,
    PteNotFound,
    OutOfMemory,
    ProcessNotFound,
    MessageBoxIsFull,
    InvalidUserAddress,
    UnknownCapType,
    UnexpectedCapType
}

pub type KernelResult<T> = Result<T, Err>;

