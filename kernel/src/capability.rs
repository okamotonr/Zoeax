use crate::{
    common::{Err, KernelResult}, address::PhysAddr, println, vm::KernelVAddress
};

use core::mem;
use core::ops::{Deref, DerefMut};

pub mod cnode;
pub mod endpoint;
pub mod notification;
pub mod tcb;
pub mod untyped;
pub mod page_table;

const CAP_TYPE_BIT: usize = 0x1f << 59;
const PADDING_BIT: usize = 0x7ff << 48;
const ADDRESS_BIT: usize = !(CAP_TYPE_BIT | PADDING_BIT);

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default)]
pub struct RawCapability([usize; 2]);

/*
 * RawCapability[1]
 * | cap_type | padding | address or none |
 * 64   5         11       48
 */

impl RawCapability {
    pub const fn null() -> Self {
        Self([0; 2])
    }

    pub fn is_null(&self) -> bool {
        if let Ok(CapabilityType::Null) = self.get_cap_type() {
            true
        } else {
            false
        }
    }
}

impl Deref for RawCapability {
    type Target = [usize; 2];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RawCapability {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RawCapability {
    pub fn get_cap_type(&self) -> KernelResult<CapabilityType> {
        CapabilityType::try_from_u8(((&self[1] & CAP_TYPE_BIT) >> 59) as u8)
    }

    pub fn set_cap_dep_val(&mut self, val: usize) {
        self[0] = val;
    }

    pub fn get_address(&self) -> PhysAddr {
        PhysAddr::new(self[1] & ADDRESS_BIT)
    }

    pub fn set_cap_type(&mut self, cap_type: CapabilityType) {
        self[1] = (self[1] & !CAP_TYPE_BIT) | ((cap_type as u8 as usize) << 59)
    }

    pub fn set_address(&mut self, address: PhysAddr) {
        self[1] = (self[1] & CAP_TYPE_BIT) | <PhysAddr as Into<usize>>::into(address)
    }

    pub fn set_address_and_type(&mut self, address: PhysAddr, cap_type: CapabilityType) {
        let v = ((cap_type as u8 as usize) << 59) | <PhysAddr as Into<usize>>::into(address);
        self[1] = v
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum CapabilityType {
    Null = 0,
    Untyped = 1,
    TCB = 3,
    EndPoint = 5,
    CNode = 7,
    Notification = 9,
    // Arch
    PageTable = 2,
    Page = 4
}

impl CapabilityType {
    pub fn try_from_u8(val: u8) -> KernelResult<Self> {
        match val {
            0 => Ok(Self::Null),
            1 => Ok(Self::Untyped),
            3 => Ok(Self::TCB),
            5 => Ok(Self::EndPoint),
            7 => Ok(Self::CNode),
            9 => Ok(Self::Notification),
            2 => Ok(Self::PageTable),
            4 => Ok(Self::Page),
            _ => Err(Err::UnknownCapType),
        }
    }
}

pub trait Capability
where
    Self: Sized,
{
    type KernelObject<'x>;
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
            Err(Err::UnexpectedCapType)
        }
    }
    fn create_raw_cap(addr: KernelVAddress) -> RawCapability {
        let mut raw_capability = RawCapability::null();
        raw_capability.set_address_and_type(addr.into(), Self::CAP_TYPE);
        raw_capability
    }
    fn create_cap_dep_val(_addr: KernelVAddress, _user_size: usize) -> usize {
        0
    }
    fn get_object_size<'a>(_user_size: usize) -> usize {
        mem::size_of::<Self::KernelObject<'a>>()
    }
    fn can_be_retyped_from_device_memory() -> bool {
        false
    }
    fn new(raw_cap: RawCapability) -> Self;
    fn get_raw_cap(&self) -> RawCapability;
    fn init_object(&mut self) -> ();
}
