use crate::{capability::{self, RawCapability}, common::KernelResult};

// TODO: Import some whare
pub struct ManagementDB([usize; 2]);

pub struct CNodeEntry {
    cap: RawCapability,
    mdb: ManagementDB
}

impl CNodeEntry {
    pub const fn null() -> Self {
        Self {
            cap: RawCapability::null(),
            mdb: ManagementDB([0; 2])
        }
    }
}

pub struct CNode<const SIZE: usize>([CNodeEntry; SIZE]);

impl<const SIZE: usize> CNode<SIZE> {
    pub fn new() -> Self {
        Self(core::array::from_fn(|_| CNodeEntry::null()))
    }

    pub fn lookup_entry(&mut self, index: u64) -> KernelResult<&mut CNodeEntry> {
        todo!()
    }

    pub fn insert_cap(&mut self, cap: RawCapability, index: u64) -> KernelResult<()> {
        todo!()
    } 
}

