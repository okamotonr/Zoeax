use super::{Capability, CapabilityType, RawCapability};
use crate::address::KernelVAddress;
use crate::common::{ErrKind, KernelResult};
use crate::kerr;
use crate::object::{CNode, CNodeEntry};

use core::mem;

/*
 * RawCapability[0]
 * |   radix    |
 * 64    64    0
 */
pub struct CNodeCap(RawCapability);
impl Capability for CNodeCap {
    const CAP_TYPE: CapabilityType = CapabilityType::CNode;
    type KernelObject = CNode;
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
    fn derive(&self, _src_slot: &CNodeEntry) -> KernelResult<Self> {
        // unchecked
        Ok(Self::new(self.get_raw_cap()))
    }

    fn init_object(&mut self) {}
}

impl CNodeCap {
    #[allow(unused_variables)]
    pub fn insert_cap(
        &mut self,
        src_slot: &mut CNodeEntry,
        new_cap: RawCapability,
        index: usize,
    ) -> KernelResult<()> {
        todo!();
    }

    pub fn get_cnode(&mut self, num: usize, offset: usize) -> KernelResult<&mut CNode> {
        (self.get_entry_num() >= num + offset)
            .then_some(())
            .ok_or(kerr!(ErrKind::NoEnoughSlot))?;
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
            let entry = cnode.lookup_entry_mut(i)?;
            entry
                .as_ref()
                .map_or(Ok(()), |_| Err(kerr!(ErrKind::NotEmptySlot)))?;
        }
        Ok(cnode)
    }

    pub fn get_src_and_dest(
        &mut self,
        src: usize,
        dst: usize,
        num: usize,
    ) -> KernelResult<(&mut CNodeEntry, &mut CNode)> {
        // TODO: check src and dst is acceptable
        (!((dst..dst + num).contains(&src)))
            .then_some(())
            .ok_or(kerr!(ErrKind::InvalidOperation))?;
        let ptr: KernelVAddress = self.0.get_address().into();
        let ptr: *mut CNodeEntry = ptr.into();
        unsafe {
            let src = ptr.add(src);
            let dst = ptr.add(dst);
            Ok((&mut *src, &mut *(dst as *mut CNode)))
        }
    }

    fn get_entry_num(&self) -> usize {
        self.radix()
    }

    pub fn lookup_entry_mut(&mut self, index: usize) -> KernelResult<&mut Option<CNodeEntry>> {
        self.get_cnode(1, index)?.lookup_entry_mut(0)
    }

    fn radix(&self) -> usize {
        self.0.cap_dep_val as usize
    }
}
