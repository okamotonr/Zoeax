use crate::{syscall::{untyped_retype, SysCallFailed}, IPCBuffer};
use shared::{cap_type::CapabilityType, err_kind::ErrKind};

pub trait KernelObject: Default {
    const CAP_TYPE: CapabilityType;
}

pub trait FixedSizeType: KernelObject {
    const OBJECT_SIZE: usize;
}

pub struct Capability<K: KernelObject> {
    pub cap_ptr: usize,
    pub cap_depth: u32,
    pub cap_data: K,
}

#[derive(Default, Debug)]
pub struct Untyped {
    pub is_device: bool,
    pub size_bits: usize,
}

impl KernelObject for Untyped {
    const CAP_TYPE: CapabilityType = CapabilityType::Untyped;
}


#[derive(Debug, Default)]
pub struct CNode {
    pub radix: u32,
    // TODO: We have to track which slots are now in using.
    // Box<[Option<&Cap<Something>; 2_usize.pow(self.radix)]
    // or simple bitmap
    // Box<[bool; 2_usize.pow(self.radix)]
    pub cursor: usize,
}


impl KernelObject for CNode {
    const CAP_TYPE: CapabilityType = CapabilityType::CNode;
}

#[derive(Debug, Default)]
pub struct PageTable {
    pub mapped_address: usize,
    pub is_root: bool,
    pub is_mapped: bool
}

impl KernelObject for PageTable {
    const CAP_TYPE: CapabilityType = CapabilityType::PageTable;
}

impl FixedSizeType for PageTable {
    const OBJECT_SIZE: usize = 4096;
}


#[derive(Debug, Default)]
pub struct Page {
    pub mapped_address: usize,
    pub is_mapped: bool,
    pub rights: u8
}

impl KernelObject for Page {
    const CAP_TYPE: CapabilityType = CapabilityType::Page;
}

impl FixedSizeType for Page {
    const OBJECT_SIZE: usize = 4096;
}

#[derive(Debug, Default)]
pub struct EndpointData {
}

impl KernelObject for EndpointData {
    const CAP_TYPE: CapabilityType = CapabilityType::EndPoint;
}

#[derive(Debug, Default)]
pub struct NotificaitonData {}

impl KernelObject for NotificaitonData {
    const CAP_TYPE: CapabilityType = CapabilityType::Notification;
}

#[derive(Debug, Default)]
pub struct TCBData {}

impl KernelObject for TCBData {
    const CAP_TYPE: CapabilityType = CapabilityType::Tcb;
}

pub type UntypedCapability = Capability<Untyped>;

pub struct CSlots {
    // TODO: Get pptr and depth from parent: &CNode
    pptr: usize,
    depth: u32,
    index: u32,
    num: u32,
}

pub struct CSlot {
    pptr: usize,
    depth: u32,
    index: u32
}

impl UntypedCapability {
    pub fn retype_mul<T>(
        &mut self,
        slots: &CSlots,
        user_size: u32,
        buffer: &mut [Capability<T>],
        num: u32,
    ) -> Result<(), SysCallFailed>
    where
        T: KernelObject,
    {
        untyped_retype(
            self.cap_ptr,
            self.cap_depth,
            slots.pptr,
            slots.depth,
            slots.index,
            user_size,
            slots.num,
            T::CAP_TYPE,
        )?;
        for i in 0..num {
            let new_c = T::default();
            buffer[i as usize] = Capability {
                cap_ptr: slots.pptr,
                cap_depth: slots.depth,
                cap_data: new_c,
            }
        }
        Ok(())
    }

    pub fn retype_single<T: KernelObject>(
        &mut self,
        slot: &mut CSlot,
        user_size: usize,
    ) -> Result<Capability<T>, SysCallFailed> {
        let num = 1;
        untyped_retype(
            self.cap_ptr,
            self.cap_depth,
            slot.pptr,
            slot.depth,
            slot.index,
            user_size as u32,
            num,
            T::CAP_TYPE,
        )?;
        let new_c = T::default();
        // We have to caluculate new cap postion.
        Ok(Capability {
            cap_ptr: slot.pptr,
            cap_depth: slot.depth,
            cap_data: new_c,
        })
    }

    pub fn retype_single_with_fixed_size<T: KernelObject + FixedSizeType>(
        &mut self,
        slot: &mut CSlot
    ) -> Result<Capability<T>, SysCallFailed> {
        // NOTE: user_size will be ignored in kernel.
        let user_size = T::OBJECT_SIZE;
        self.retype_single::<T>(slot, user_size)
    }
}

pub type CNodeCapability = Capability<CNode>;

impl CNodeCapability {

    pub fn get_slot(&mut self) -> Result<CSlot, SysCallFailed> {
        let size = self.get_size();
        if self.cap_data.cursor >= size {
            Err((ErrKind::NoEnoughSlot, 0))
        } else {
            let ret = Ok(CSlot {
                pptr: self.cap_ptr,
                depth: self.cap_depth,
                index: self.cap_data.cursor as u32
            });
            self.cap_data.cursor += 1;
            ret
        }
    }

    pub fn get_size(&self) -> usize {
        2_usize.pow(self.cap_data.radix)
    }
}

pub type PageTableCapability = Capability<PageTable>;

impl PageTableCapability {
    pub fn map(&mut self, root_table: &mut Self, vaddr: usize) -> Result<usize, SysCallFailed> {
        todo!("")
    }
}

pub type PageCapability = Capability<Page>;

impl PageCapability {
    pub fn map(&mut self, root_table: &mut PageTableCapability, vaddr: usize, flags: PageFlags) -> Result<(), SysCallFailed> {
        todo!()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PageFlags {
    pub is_writable: bool,
    pub is_readable: bool,
    pub is_executable: bool
}

pub type TCBCapability = Capability<TCBData>;
