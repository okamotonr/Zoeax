use crate::{syscall::{untyped_retype, SysCallFailed}, IPCBuffer};
use shared::cap_type::CapabilityType;

pub struct Zoea {
    pub ipc_message: &'static IPCBuffer,
}

pub trait Cap: Default {
    const CAP_TYPE: CapabilityType;
}

pub struct Capability<T: Cap> {
    pub cap_ptr: usize,
    pub cap_depth: u32,
    pub cap_data: T,
}

#[derive(Default, Debug)]
pub struct UntypedData {
    pub is_device: bool,
    pub size_bits: usize,
}

impl Cap for UntypedData {
    const CAP_TYPE: CapabilityType = CapabilityType::Untyped;
}


#[derive(Debug, Default)]
pub struct CNodeData {
    pub radix: u32,
    // TODO: We have to track which slots are now in using.
    // Box<[Option<&Cap<Something>; 2_usize.pow(self.radix)]
    // or simple bitmap
    // Box<[bool; 2_usize.pow(self.radix)]
}

impl Cap for CNodeData {
    const CAP_TYPE: CapabilityType = CapabilityType::CNode;
}

#[derive(Debug, Default)]
pub struct PageTableData {
    pub mapped_address: usize,
    pub is_root: bool,
    pub is_mapped: bool
}

impl Cap for PageTableData {
    const CAP_TYPE: CapabilityType = CapabilityType::PageTable;
}

#[derive(Debug, Default)]
pub struct PageData {
    pub mapped_address: usize,
    pub is_mapped: bool,
    pub rights: u8
}

impl Cap for PageData {
    const CAP_TYPE: CapabilityType = CapabilityType::Page;
}

#[derive(Debug, Default)]
pub struct EndpointData {
}

impl Cap for EndpointData {
    const CAP_TYPE: CapabilityType = CapabilityType::EndPoint;
}

#[derive(Debug, Default)]
pub struct NotificaitonData {}

impl Cap for NotificaitonData {
    const CAP_TYPE: CapabilityType = CapabilityType::Notification;
}

#[derive(Debug, Default)]
pub struct TCBData {}

impl Cap for TCBData {
    const CAP_TYPE: CapabilityType = CapabilityType::Tcb;
}

pub type UntypedCapability = Capability<UntypedData>;

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
        T: Cap,
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

    pub fn retype_single<T: Cap>(
        &mut self,
        slot: &CSlot,
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
        Ok(Capability {
            cap_ptr: slot.pptr,
            cap_depth: slot.depth,
            cap_data: new_c,
        })
    }
}

pub type CNodeCapability = Capability<CNodeData>;

impl CNodeCapability {
}

pub type PageTableCapability = Capability<PageTableData>;

pub type PageCapability = Capability<PageData>;

pub type TCBCapability = Capability<TCBData>;
