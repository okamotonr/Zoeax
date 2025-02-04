use super::{Capability, CapabilityData, CapabilityType, Something};
use crate::address::KernelVAddress;
use crate::common::{ErrKind, KernelResult};
use crate::object::{CNode, CNodeEntry, KObject};
use crate::{kerr, print, println};

use core::mem;

/*
 * RawCapability[0]
 * | padding |  radix  |
 * 63      32         0
 */
impl KObject for CNode {}

pub type CNodeCap = CapabilityData<CNode>;

impl Capability for CNodeCap {
    const CAP_TYPE: CapabilityType = CapabilityType::CNode;
    type KernelObject = CNode;
    fn create_cap_dep_val(_addr: KernelVAddress, user_size: usize) -> usize {
        user_size
    }

    fn get_object_size<'a>(user_size: usize) -> usize {
        2_usize.pow(user_size as u32) * mem::size_of::<CNodeEntry<Something>>()
    }
    fn derive(&self, _src_slot: &CNodeEntry<Something>) -> KernelResult<Self> {
        // unchecked
        Ok(Self { ..*self })
    }

    fn init_object(&mut self) {
        // TODO: Zero clear
    }
}

impl CNodeCap {
    pub fn get_cnode(&mut self) -> &mut [Option<CNodeEntry<Something>>] {
        let ptr: KernelVAddress = self.get_address().into();
        let ptr: *mut Option<CNodeEntry<Something>> = ptr.into();
        unsafe { core::slice::from_raw_parts_mut(ptr, 2_usize.pow(self.radix())) }
    }

    pub fn get_cnode_ref(&self) -> &[Option<CNodeEntry<Something>>] {
        let ptr: KernelVAddress = self.get_address().into();
        let ptr: *const Option<CNodeEntry<Something>> = ptr.into();
        unsafe { core::slice::from_raw_parts(ptr, 2_usize.pow(self.radix())) }
    }
    pub fn get_src_and_dest(
        &mut self,
        src: usize,
        dst: usize,
        num: usize,
    ) -> KernelResult<(&mut CNodeEntry<Something>, &mut CNode)> {
        // TODO: check src and dst is acceptable
        (!((dst..dst + num).contains(&src)))
            .then_some(())
            .ok_or(kerr!(ErrKind::InvalidOperation))?;
        let ptr: KernelVAddress = self.get_address().into();
        let ptr: *mut CNodeEntry<Something> = ptr.into();
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
    ) -> KernelResult<&mut Option<CNodeEntry<Something>>> {
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
                        let ptr = cap as *mut CapabilityData<Something> as *mut CNodeCap;
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
    ) -> KernelResult<&mut Option<CNodeEntry<Something>>> {
        self.lookup_entry_mut(capptr, self.radix())
    }

    fn _lookup_entry_mut(
        &mut self,
        capptr: usize,
        depth_bits: u32,
    ) -> KernelResult<(&mut Option<CNodeEntry<Something>>, u32)> {
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
        self.cap_dep_val as u32
    }
    /// debug perpsoe
    pub fn print_traverse(&self) {
        self.print_level(0)
    }

    fn print_level(&self, level: usize) {
        let c_node = self.get_cnode_ref();
        for _ in 0..level {
            print!("  ");
        }
        print!("|");
        println!("level is {}, radix is {}", level, self.radix());
        for (i, slot) in c_node.iter().enumerate() {
            if let Some(ref entry) = slot {
                for _ in 0..level {
                    print!("   ");
                }
                print!("|");
                // TODO: Prity print
                print!("level is {}, index is {}, {:?}", level, i, entry);
                if let Ok(cap) = entry.cap().as_capability_ref::<CNode>() {
                    if cap.get_address() == self.get_address() {
                        println!("  # same cnode of current");
                    } else {
                        cap.print_level(level + 1)
                    };
                } else {
                    print!("\n");
                }
            }
        }
    }
}
