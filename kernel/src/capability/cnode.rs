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
        2_usize.pow(user_size as u32) * mem::size_of::<CNodeEntry>()
    }
    fn derive(&self, _src_slot: &CNodeEntry) -> KernelResult<Self> {
        // unchecked
        Ok(Self::new(self.get_raw_cap()))
    }

    fn init_object(&mut self) {
        // TODO: Zero clear
    }
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

    pub fn get_cnode(&mut self) -> &mut [Option<CNodeEntry>] {
        let ptr: KernelVAddress = self.0.get_address().into();
        let ptr: *mut Option<CNodeEntry> = ptr.into();
        unsafe { core::slice::from_raw_parts_mut(ptr, 2_usize.pow(self.radix())) }
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

    pub fn lookup_entry_mut(
        &mut self,
        capptr: usize,
        depth_bits: u32,
    ) -> KernelResult<&mut Option<CNodeEntry>> {
        let mut cnode_cap = self;
        let mut depth_bits = depth_bits;
        loop {
            let (next_cap, next_bits) = match cnode_cap._lookup_entry_mut(capptr, depth_bits)? {
                (val @ &mut None, _) => return Ok(val),
                (val, 0) => return Ok(val),
                (val, rem) => {
                    let entry = val.as_mut().unwrap();
                    let cap = entry.cap_ref_mut();
                    if cap.get_cap_type()? != CapabilityType::CNode {
                        return Ok(val);
                    }
                    unsafe {
                        // TODO: Fix this dirty hack
                        let ptr = cap as *mut RawCapability as *mut CNodeCap;
                        (&mut *ptr, rem)
                    }
                }
            };
            cnode_cap = next_cap;
            depth_bits = next_bits;
        }
    }

    pub fn lookup_entry_mut_one_level(
        &mut self,
        capptr: usize,
    ) -> KernelResult<&mut Option<CNodeEntry>> {
        self.lookup_entry_mut(capptr, self.radix())
    }

    fn _lookup_entry_mut(
        &mut self,
        capptr: usize,
        depth_bits: u32,
    ) -> KernelResult<(&mut Option<CNodeEntry>, u32)> {
        let radix = self.radix();
        let remain_bits = depth_bits
            .checked_sub(radix)
            .ok_or(kerr!(ErrKind::OutOfMemory))?;
        let cnode = self.get_cnode();
        let offset = (capptr >> remain_bits) & ((1 << radix) - 1); // TODO: usize::BITS
        let entry = &mut cnode[offset];
        Ok((entry, remain_bits))
    }

    fn radix(&self) -> u32 {
        self.0.cap_dep_val as u32
    }
}
