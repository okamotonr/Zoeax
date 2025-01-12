use crate::{
    address::{KernelVAddress, PhysAddr},
    capability::RawCapability,
    common::KernelResult,
};
use core::mem;

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
#[derive(Default, Debug)]
pub struct ManagementDB([usize; 2]);

impl ManagementDB {
    pub fn get_next(&mut self) -> Option<&mut CNodeEntry> {
        self.get_node(true)
    }

    pub fn set_next(&mut self, next: &mut CNodeEntry) {
        self.set_node(next, true)
    }

    #[allow(dead_code)]
    pub fn get_prev(&mut self) -> Option<&mut CNodeEntry> {
        self.get_node(false)
    }

    #[allow(dead_code)]
    pub fn set_prev(&mut self, prev: &mut CNodeEntry) {
        self.set_node(prev, false)
    }

    fn get_node(&mut self, is_next: bool) -> Option<&mut CNodeEntry> {
        let index = if is_next { 1 } else { 0 };
        let address = (self.0[index] >> 16) as *const CNodeEntry;
        if address.is_null() {
            None
        } else {
            let k_address: *mut CNodeEntry = KernelVAddress::from(PhysAddr::from(address)).into();
            unsafe { k_address.as_mut() }
        }
    }

    unsafe fn get_entry(&mut self) -> *mut CNodeEntry {
        let offset = mem::offset_of!(CNodeEntry, mdb);
        (self as *mut ManagementDB)
            .byte_sub(offset)
            .cast::<CNodeEntry>()
    }

    fn set_node(&mut self, node: &mut CNodeEntry, is_next: bool) {
        self._set_node(is_next, node);
        let parent = unsafe { self.get_entry().as_mut().unwrap() };
        node.mdb._set_node(!is_next, parent);
    }

    fn _set_node(&mut self, is_next: bool, node: &CNodeEntry) {
        let index = if is_next { 1 } else { 0 };
        self.0[index] &= 0xffff;
        self.0[index] |= (node as *const CNodeEntry as usize) << 16;
    }
}

#[derive(Debug, Default)]
pub struct CNodeEntry {
    cap: RawCapability,
    mdb: ManagementDB,
}

impl CNodeEntry {
    pub const fn null() -> Self {
        Self {
            cap: RawCapability::null(),
            mdb: ManagementDB([0; 2]),
        }
    }

    pub fn new_with_rawcap(cap: RawCapability) -> Self {
        Self {
            cap,
            mdb: ManagementDB([0; 2]),
        }
    }

    pub fn cap(&self) -> RawCapability {
        self.cap
    }

    pub fn insert(&mut self, parent: &mut Self, cap: RawCapability) {
        self.cap = cap;
        if let Some(prev_next) = parent.get_next() {
            self.set_next(prev_next);
        };
        parent.set_next(self)
    }

    pub fn set_next(&mut self, next: &mut Self) {
        self.mdb.set_next(next)
    }

    pub fn get_next(&mut self) -> Option<&mut Self> {
        self.mdb.get_next()
    }

    pub fn set_cap(&mut self, raw_cap: RawCapability) {
        self.cap = raw_cap
    }

    pub fn is_null(&self) -> bool {
        self.cap.is_null()
    }
}

#[derive(Debug, Default)]
pub struct CNode;

impl CNode {
    pub fn new() -> Self {
        Self
    }

    pub fn lookup_entry(&mut self, index: usize) -> KernelResult<&mut CNodeEntry> {
        let root = (self as *mut Self).cast::<CNodeEntry>();
        unsafe {
            let ret = root.add(index);
            Ok(ret.as_mut().unwrap())
        }
    }

    pub fn insert_cap(&mut self, parent: &mut CNodeEntry, cap: RawCapability, index: usize) {
        let root = (self as *mut Self).cast::<CNodeEntry>();
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
