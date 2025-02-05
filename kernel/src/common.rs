use core::convert::TryFrom;
use core::{error::Error, fmt, mem};

use crate::Registers;

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
#[macro_export]
macro_rules! const_assert_single {
    ($cond:expr, $msg:expr $(,)?) => {
        const _: () = {
            if !$cond {
                panic!($msg);
            }
        };
    };
    ($cond:expr $(,)?) => {
        const _: () = {
            if !$cond {
                panic!(concat!(
                    "Compile-time assertion failed: ",
                    stringify!($cond)
                ));
            }
        };
    };
}

#[macro_export]
macro_rules! const_assert {
    ($($cond:expr),+ $(,)?) => {
        $( $crate::const_assert_single!($cond); )+
    };
    ( $( $cond:expr => $msg:expr ),+ $(,)? ) => {
        $( $crate::const_assert_single!($cond, $msg); )+
    };
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
    PageTableNotMappedYet,
    PageAlreadyMapped,
    PageNotMappedYet,
    UnknownInvocation,
    CanNotDerivable,
    InvalidOperation,
    CapNotFound,
    NotAligned,
    UnknownSysCall,
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
            e_val if e_val == ErrKind::PageNotMappedYet as usize => Ok(ErrKind::PageNotMappedYet),
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

const_assert!(mem::size_of::<BootInfo>() <= 4096);

// bits, idx, is_device
#[derive(Default, Debug)]
pub struct UntypedInfo {
    pub bits: usize,
    pub idx: usize,
    pub is_device: bool,
}

#[derive(Default, Debug)]
pub struct BootInfo {
    pub ipc_buffer_addr: usize,
    pub root_cnode_idx: usize,
    pub root_vspace_idx: usize,
    pub untyped_num: usize,
    pub firtst_empty_idx: usize,
    pub msg: [u8; 32],
    pub untyped_infos: [UntypedInfo; 32],
}

impl BootInfo {
    #[allow(clippy::mut_from_ref)]
    pub fn ipc_buffer(&self) -> &mut IPCBuffer {
        let ptr = self.ipc_buffer_addr as *mut IPCBuffer;
        unsafe { ptr.as_mut().unwrap() }
    }
}

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

pub const MESSAGE_LEN: usize = 128;

pub struct IPCBuffer {
    pub tag: usize,
    pub message: [usize; MESSAGE_LEN],
    pub user_data: usize,
}

impl IPCBuffer {
    pub fn write_as<F, T>(&mut self, write_fn: F) -> Result<(), ErrKind>
    where
        F: FnOnce() -> T,
        T: Sized,
    {
        (size_of_val(&self.message) >= mem::size_of::<T>())
            .then_some(())
            .ok_or(ErrKind::InvalidOperation)?;
        let ptr = &mut self.message[0] as *mut usize as *mut T;
        unsafe {
            *ptr = write_fn();
        }
        Ok(())
    }

    pub fn read_as<T: Sized>(&self) -> Result<&T, ErrKind> {
        (size_of_val(&self.message) >= mem::size_of::<T>())
            .then_some(())
            .ok_or(ErrKind::InvalidOperation)?;
        let ptr = &self.message[0] as *const usize as *const T;
        unsafe { Ok(ptr.as_ref().unwrap()) }
    }
}

const_assert!(
    mem::size_of::<BootInfo>() <= 4096,
    mem::size_of::<IPCBuffer>() <= 4096
);
const_assert!(mem::size_of::<Registers>() <= mem::size_of::<[usize; MESSAGE_LEN]>());
