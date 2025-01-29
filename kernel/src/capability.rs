use crate::{
    address::{KernelVAddress, PhysAddr},
    common::{ErrKind, KernelResult},
    kerr,
    object::CNodeEntry,
};

use core::num::NonZeroU8;
use core::{fmt, mem};

pub mod cnode;
pub mod endpoint;
pub mod notification;
pub mod page_table;
pub mod tcb;
pub mod untyped;

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct RawCapability {
    cap_type: NonZeroU8,
    cap_right: u8,
    address_top: u16,
    address_bottom: u32,
    cap_dep_val: u64,
}

/*
 * RawCapability[1]
 * | cap_type | cap_right | padding | address or none |
 * 64   5         5           6              48
 */

impl RawCapability {
    pub fn new(cap_type: CapabilityType, address: PhysAddr) -> Self {
        let mut ret = Self {
            cap_type: NonZeroU8::new(cap_type as u8).unwrap(),
            cap_right: 0,
            address_top: 0,
            address_bottom: 0,
            cap_dep_val: 0,
        };
        ret.set_address(address);
        ret
    }
    pub fn get_cap_type(&self) -> KernelResult<CapabilityType> {
        CapabilityType::try_from_u8(self.cap_type.get())
    }

    // TODO: u64 and usize
    pub fn set_cap_dep_val(&mut self, val: usize) {
        self.cap_dep_val = val as u64;
    }

    pub fn get_address(&self) -> PhysAddr {
        // TODO: u64 and usize
        let addr = (((self.address_top as u64) << 32) | self.address_bottom as u64) as usize;
        PhysAddr::new(addr)
    }

    pub fn set_cap_type(&mut self, cap_type: CapabilityType) {
        self.cap_type = NonZeroU8::new(cap_type as u8).unwrap()
    }

    pub fn set_address(&mut self, address: PhysAddr) {
        let address: usize = address.into();
        let address_top = ((address >> 32) & u16::MAX as usize) as u16;
        let address_bottom = (address & u32::MAX as usize) as u32;
        self.address_top = address_top;
        self.address_bottom = address_bottom
    }

    pub fn set_address_and_type(&mut self, address: PhysAddr, cap_type: CapabilityType) {
        self.set_cap_type(cap_type);
        self.set_address(address)
    }
}

impl fmt::Debug for RawCapability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "captype: {:?}, address: {:?}",
            self.get_cap_type(),
            self.get_address()
        )
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum CapabilityType {
    Untyped = 1,
    TCB = 3,
    EndPoint = 5,
    CNode = 7,
    Notification = 9,
    // Arch
    PageTable = 2,
    Page = 4,
}

impl CapabilityType {
    pub fn try_from_u8(val: u8) -> KernelResult<Self> {
        match val {
            1 => Ok(Self::Untyped),
            3 => Ok(Self::TCB),
            5 => Ok(Self::EndPoint),
            7 => Ok(Self::CNode),
            9 => Ok(Self::Notification),
            2 => Ok(Self::PageTable),
            4 => Ok(Self::Page),
            _ => Err(kerr!(ErrKind::UnknownCapType)),
        }
    }
}

// TODO: Change of capability should change raw_cap in slot.
pub trait Capability
where
    Self: Sized,
{
    type KernelObject;
    const CAP_TYPE: CapabilityType;

    fn init(addr: KernelVAddress, user_size: usize) -> Self {
        let mut raw_cap = Self::create_raw_cap(addr);
        let val = Self::create_cap_dep_val(addr, user_size);
        raw_cap.set_cap_dep_val(val);
        Self::new(raw_cap)
    }
    fn try_from_raw(raw_cap: RawCapability) -> KernelResult<Self> {
        let cap_type = raw_cap.get_cap_type()?;
        if cap_type == Self::CAP_TYPE {
            Ok(Self::new(raw_cap))
        } else {
            Err(kerr!(ErrKind::UnexpectedCapType))
        }
    }
    fn create_raw_cap(addr: KernelVAddress) -> RawCapability {
        RawCapability::new(Self::CAP_TYPE, addr.into())
    }
    fn create_cap_dep_val(_addr: KernelVAddress, _user_size: usize) -> usize {
        0
    }
    fn get_object_size(_user_size: usize) -> usize {
        mem::size_of::<Self::KernelObject>()
    }
    fn can_be_retyped_from_device_memory() -> bool {
        false
    }
    fn derive(&self, _src_slot: &CNodeEntry) -> KernelResult<Self> {
        Err(kerr!(ErrKind::CanNotDerivable))
    }
    fn new(raw_cap: RawCapability) -> Self;
    fn get_raw_cap(&self) -> RawCapability;
    fn init_object(&mut self);
}
