use crate::{
    address::{KernelVAddress, PhysAddr},
    capability::{CapInSlot, Capability, CapabilityData, Something},
    common::KernelResult,
    const_assert, CapabilityType,
};
use core::{fmt::Debug, mem};

use super::KObject;

/*
 * ManagementDB[0]
 * |  prev node entry | padding |
 * 64                          0
 *       48               16
 * ManagementDB[1]
 * | next node entry  | padding |
 * 64                          0
 *       48               16
 */

const_assert!(mem::size_of::<CNodeEntry<Something>>() == mem::size_of::<CSlot>());

#[derive(Default, Debug)]
pub struct ManagementDB([usize; 2]);

impl ManagementDB {
    pub fn get_next(&mut self) -> Option<&mut CNodeEntry<Something>> {
        self.get_node(true)
    }

    pub fn set_next(&mut self, next: &mut CNodeEntry<Something>) {
        self.set_node(next, true)
    }

    #[allow(dead_code)]
    pub fn get_prev(&mut self) -> Option<&mut CNodeEntry<Something>> {
        self.get_node(false)
    }

    #[allow(dead_code)]
    pub fn set_prev(&mut self, prev: &mut CNodeEntry<Something>) {
        self.set_node(prev, false)
    }

    fn get_node(&mut self, is_next: bool) -> Option<&mut CNodeEntry<Something>> {
        let index = if is_next { 1 } else { 0 };
        let address = (self.0[index] >> 16) as *const CNodeEntry<Something>;
        if address.is_null() {
            None
        } else {
            let k_address: *mut CNodeEntry<Something> =
                KernelVAddress::from(PhysAddr::from(address)).into();
            unsafe { k_address.as_mut() }
        }
    }

    unsafe fn get_entry(&mut self) -> *mut CNodeEntry<Something> {
        let offset = mem::offset_of!(CNodeEntry<Something>, mdb);
        (self as *mut ManagementDB)
            .byte_sub(offset)
            .cast::<CNodeEntry<Something>>()
    }

    fn set_node(&mut self, node: &mut CNodeEntry<Something>, is_next: bool) {
        self._set_node(is_next, node);
        let parent = unsafe { self.get_entry().as_mut().unwrap() };
        node.mdb._set_node(!is_next, parent);
    }

    fn _set_node(&mut self, is_next: bool, node: &CNodeEntry<Something>) {
        let index = if is_next { 1 } else { 0 };
        self.0[index] &= 0xffff;
        self.0[index] |= (node as *const CNodeEntry<Something> as usize) << 16;
    }
}

pub type CSlot<T = Something> = Option<CNodeEntry<T>>;

#[derive(Debug)]
pub struct CNodeEntry<K: KObject>
where
    K: KObject,
    CapabilityData<K>: Capability,
{
    cap: CapabilityData<K>,
    mdb: ManagementDB,
}

impl CNodeEntry<Something> {
    pub fn as_capability<K>(&mut self) -> KernelResult<&mut CNodeEntry<K>>
    where
        K: KObject,
        CapabilityData<K>: Capability,
    {
        // whether cast is safe or
        self.cap.as_capability::<K>()?;
        unsafe {
            let ptr = self as *mut Self as *mut CNodeEntry<K>;
            Ok(ptr.as_mut().unwrap())
        }
    }
    pub fn get_cap_type(&self) -> KernelResult<CapabilityType> {
        self.cap.get_cap_type()
    }
}

impl<K: KObject> CNodeEntry<K>
where
    CapabilityData<K>: Capability,
{
    pub fn new_with_rawcap(cap: CapabilityData<K>) -> Self {
        Self {
            cap,
            mdb: ManagementDB([0; 2]),
        }
    }

    pub fn cap_and_mdb(&mut self) -> (&mut CapabilityData<K>, &mut ManagementDB) {
        (&mut self.cap, &mut self.mdb)
    }

    pub fn cap(&self) -> &CapabilityData<K> {
        &self.cap
    }

    pub fn insert(&mut self, parent: &mut CNodeEntry<Something>) {
        if let Some(prev_next) = parent.get_next() {
            self.set_next(prev_next);
        };
        parent.set_next(self.up_cast_ref_mut())
    }

    pub fn replace(&mut self, src: &mut CNodeEntry<Something>) {
        if let Some(src_next) = src.get_next() {
            src_next.set_prev(self.up_cast_ref_mut());
            self.set_next(src_next);
        };
        if let Some(src_prev) = src.get_prev() {
            src_prev.set_next(self.up_cast_ref_mut());
            self.set_prev(src_prev);
        }
    }

    pub fn set_next(&mut self, next: &mut CNodeEntry<Something>) {
        self.mdb.set_next(next)
    }

    pub fn set_prev(&mut self, prev: &mut CNodeEntry<Something>) {
        self.mdb.set_prev(prev)
    }

    pub fn get_next(&mut self) -> Option<&mut CNodeEntry<Something>> {
        self.mdb.get_next()
    }

    pub fn get_prev(&mut self) -> Option<&mut CNodeEntry<Something>> {
        self.mdb.get_prev()
    }

    pub fn cap_ref_mut(&mut self) -> &mut CapabilityData<K> {
        &mut self.cap
    }

    pub fn up_cast_ref(&self) -> &CNodeEntry<Something> {
        unsafe {
            let ptr = self as *const CNodeEntry<K> as *const CNodeEntry<Something>;
            ptr.as_ref().unwrap()
        }
    }

    pub fn up_cast_ref_mut(&mut self) -> &mut CNodeEntry<Something> {
        unsafe {
            let ptr = self as *mut CNodeEntry<K> as *mut CNodeEntry<Something>;
            ptr.as_mut().unwrap()
        }
    }
}

#[derive(Debug, Default)]
pub struct CNode;

impl CNode {
    pub fn new() -> Self {
        Self
    }

    pub fn insert_cap(&mut self, parent: &mut ManagementDB, cap: CapInSlot, index: usize) {
        let root = (self as *mut Self).cast::<CNodeEntry<Something>>();
        let mut entry = CNodeEntry {
            cap,
            mdb: ManagementDB::default(),
        };
        if let Some(prev_next) = parent.get_next() {
            entry.set_next(prev_next);
        };
        parent.set_next(&mut entry);
        unsafe {
            *root.add(index) = entry;
        }
    }
}
