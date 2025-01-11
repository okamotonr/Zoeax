use super::{RawCapability, CapabilityType, Capability};
use crate::common::{KernelResult, Err};
use crate::object::{CNode, CNodeEntry};
use crate::vm::KernelVAddress;

use core::mem;

/*
 * RawCapability[0]
 * |   radix    |
 * 64    64    0
 */
pub struct CNodeCap(RawCapability);
impl Capability for CNodeCap {
    const CAP_TYPE: CapabilityType = CapabilityType::CNode;
    type KernelObject<'x> = CNode;
    fn new(raw_cap: RawCapability) -> Self {
        Self(raw_cap)
    }
    fn create_cap_dep_val(_addr: KernelVAddress, user_size: usize) -> usize {
        user_size
    }
    fn get_raw_cap(&self) -> RawCapability {
        self.0
    }

    fn get_object_size<'a>(user_size: usize) -> usize {
        user_size * mem::size_of::<CNodeEntry>()
    }
    
    fn init_object(&mut self) -> () {
        ()
    }
}

impl CNodeCap {
    pub fn insert_cap(&mut self, src_slot: &mut CNodeEntry, new_cap: RawCapability, index: usize) -> KernelResult<()> {
        todo!();
        Ok(())
    }

    pub fn get_cnode(&mut self, num: usize, offset: usize) -> KernelResult<&mut CNode> {
        (self.get_entry_num() >= num + offset).then_some(()).ok_or(Err::NoEnoughSlot)?;
        let ptr: KernelVAddress = self.0.get_address().into();
        let ptr: *mut CNodeEntry = ptr.into();
        unsafe {
            let cnode = ptr.add(offset).cast::<CNode>();
            Ok(cnode.as_mut().unwrap())
        }
    }

    pub fn get_writable(&mut self, num: usize, offset: usize) -> KernelResult<&mut CNode> {
        let cnode = self.get_cnode(num, offset)?;
        for i in 0..num {
            let entry = cnode.lookup_entry(i)?;
            (!entry.is_null()).then_some(()).ok_or(Err::NotEntrySlot)?;
        }
        Ok(cnode)
    }

    fn get_entry_num(&self) -> usize {
        self.radix()
    }

    pub fn lookup_entry(&mut self, index: usize) -> KernelResult<&mut CNodeEntry> {
        self.get_cnode(1, index)?.lookup_entry(0)
    }

    fn radix(&self) -> usize {
        self.0[0]
    }
}

