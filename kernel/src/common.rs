use core::convert::TryFrom;
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
    NoMemory = 1,
    PteNotFound,
    OutOfMemory,
    InvalidUserAddress,
    UnknownCapType,
    UnexpectedCapType,
    CanNotNewFromDeviceMemory,
    NoEnoughSlot,
    NotEmptySlot,
    SlotIsEmpty,
    VaddressAlreadyMapped,
    PageTableAlreadyMapped,
    PageAlreadyMapped,
    PageTableNotMappedYet,
    UnknownInvocation,
    CanNotDerivable,
    InvalidOperation,
    CapNotFound,
    NotAligned,
}

impl TryFrom<usize> for ErrKind {
    type Error = ();
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            e_val if e_val == ErrKind::NoMemory as usize => Ok(ErrKind::NoMemory),
            e_val if e_val == ErrKind::PteNotFound as usize => Ok(ErrKind::PteNotFound),
            e_val if e_val == ErrKind::OutOfMemory as usize => Ok(ErrKind::OutOfMemory),
            e_val if e_val == ErrKind::InvalidUserAddress as usize => {
                Ok(ErrKind::InvalidUserAddress)
            }
            e_val if e_val == ErrKind::UnknownCapType as usize => Ok(ErrKind::UnknownCapType),
            e_val if e_val == ErrKind::UnexpectedCapType as usize => Ok(ErrKind::UnexpectedCapType),
            e_val if e_val == ErrKind::CanNotNewFromDeviceMemory as usize => {
                Ok(ErrKind::CanNotNewFromDeviceMemory)
            }
            e_val if e_val == ErrKind::NoEnoughSlot as usize => Ok(ErrKind::NoEnoughSlot),
            e_val if e_val == ErrKind::NotEmptySlot as usize => Ok(ErrKind::NotEmptySlot),
            e_val if e_val == ErrKind::SlotIsEmpty as usize => Ok(ErrKind::SlotIsEmpty),
            e_val if e_val == ErrKind::VaddressAlreadyMapped as usize => {
                Ok(ErrKind::VaddressAlreadyMapped)
            }
            e_val if e_val == ErrKind::PageAlreadyMapped as usize => Ok(ErrKind::PageAlreadyMapped),
            e_val if e_val == ErrKind::PageTableAlreadyMapped as usize => {
                Ok(ErrKind::PageTableAlreadyMapped)
            }
            e_val if e_val == ErrKind::PageTableNotMappedYet as usize => {
                Ok(ErrKind::PageTableNotMappedYet)
            }
            e_val if e_val == ErrKind::UnknownInvocation as usize => Ok(ErrKind::UnknownInvocation),
            e_val if e_val == ErrKind::CanNotDerivable as usize => Ok(ErrKind::CanNotDerivable),
            e_val if e_val == ErrKind::InvalidOperation as usize => Ok(ErrKind::InvalidOperation),
            e_val if e_val == ErrKind::CapNotFound as usize => Ok(ErrKind::CapNotFound),
            e_val if e_val == ErrKind::NotAligned as usize => Ok(ErrKind::NotAligned),
            _ => Err(()),
        }
    }
}

pub type KernelResult<T> = Result<T, KernelError>;

//TODO: thiserror and anyhow
#[derive(Debug)]
pub struct KernelError {
    pub e_kind: ErrKind,
    pub e_val: u16,
    #[cfg(debug_assertions)]
    pub e_place: EPlace,
}

#[macro_export]
macro_rules! kerr {
    ($ekind:expr) => {
        $crate::common::KernelError {
            e_kind: $ekind,
            e_val: 0,
            #[cfg(debug_assertions)]
            e_place: $crate::common::EPlace {
                e_line: line!(),
                e_column: column!(),
                e_file: file!(),
            },
        }
    };

    ($ekind:expr, $eval:expr) => {
        $crate::common::KernelError {
            e_kind: $ekind,
            e_val: $eval,
            #[cfg(debug_assertions)]
            e_place: $crate::common::EPlace {
                e_line: line!(),
                e_column: column!(),
                e_file: file!(),
            },
        }
    };
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for KernelError {}

#[cfg(debug_assertions)]
#[derive(Debug)]
pub struct EPlace {
    pub e_line: u32,
    pub e_column: u32,
    pub e_file: &'static str,
}
