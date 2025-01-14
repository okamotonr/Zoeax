use core::{error::Error, fmt};

pub fn is_aligned(value: usize, align: usize) -> bool {
    value % align == 0
}

/// align should be power of 2.
pub fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

/// align should be power of 2.
pub fn align_down(value: usize, align: usize) -> usize {
    (value) & !(align - 1)
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrKind {
    NoMemory,
    TooManyTasks,
    PteNotFound,
    OutOfMemory,
    ProcessNotFound,
    MessageBoxIsFull,
    InvalidUserAddress,
    UnknownCapType,
    UnexpectedCapType,
    CanNotNewFromDeviceMemory,
    NoEnoughSlot,
    NotEntrySlot,
    VaddressAlreadyMapped,
    PageTableAlreadyMapped,
    PageAlreadyMapped,
    PageTableNotMappedYet,
    UnknownInvocation,
    CanNotDerivable,
    InvalidOperation,
}

pub type KernelResult<T> = Result<T, KernelError>;

// thiserror no_std...
#[derive(Debug)]
pub struct KernelError {
    pub e_kind: ErrKind,
    pub e_val: u16,
    // #[cfg(debug_assertions)]
    // e_place: EPlace,
}

impl From<ErrKind> for KernelError {
    fn from(value: ErrKind) -> Self {
        Self {e_kind: value, e_val: 0}
    }
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for KernelError {}
//
// #[cfg(debug_assertions)]
// #[derive(Debug)]
// pub struct EPlace {
//     e_line: u32,
//     e_column: u32,
//     e_file: &'static str
// }
