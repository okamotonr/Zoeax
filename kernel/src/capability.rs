use crate::{common::{Err, KernelResult}, memory::PhysAddr, vm::KernelVAddress};
use crate::object;

use core::ops::{DerefMut, Deref};

const CAP_TYPE_BIT: usize = 0x1f << 59;

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct RawCapability([usize; 2]);

impl RawCapability {
    pub const fn null() -> Self {
        Self([0; 2])
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
        CapabilityType::try_from_u8(
            ((&self.0[1] & CAP_TYPE_BIT) >> 59) as u8
        )
    }

}

#[repr(u8)]
#[derive(Debug)]
pub enum CapabilityType {
    Untyped,
    TCB,
    EndPoint,
    CNode
}

impl CapabilityType {
    pub fn try_from_u8(val: u8) -> KernelResult<Self> {
        match val {
            1 => Ok(Self::Untyped),
            3 => Ok(Self::TCB),
            5 => Ok(Self::EndPoint),
            7 => Ok(Self::CNode),
            _ => Err(Err::UnknownCapType)
        }
    }
}

pub trait Capability where Self: Sized {
    fn init(addr: KernelVAddress, user_size: usize) -> Self;
    fn from_raw(raw_cap: RawCapability) -> KernelResult<Self>;
    fn get_object_size(user_size: usize) -> usize;
    fn get_raw_cap(&self) -> RawCapability;
}

/*
 * | 47 bit free_idx | 10 bit padd | 1 bit is_device | 6 bit block_size |
 * 64                                                                   0
 */

pub struct UntypedCap(RawCapability);

const ADDRESS_LENGTH: usize = 47; // sv48
impl UntypedCap {
    pub fn retype<T: Capability>(&mut self, user_size: usize) -> KernelResult<T> {
        // 1, whether memory is enough or not
        // 2, write object into free memory area
        // 3, create capability of object
        // 4, update self information
        let block_size = self.block_size();
        let object_size = T::get_object_size(user_size);
        (block_size < object_size).then_some(()).ok_or(Err::NoMemory)?;
        todo!()
    }

    pub fn get_free_index(&self) -> KernelVAddress {
        let physadd = PhysAddr::new((&self.0[0] >> (64 - ADDRESS_LENGTH)) as usize);
        physadd.into()
    }

    pub fn is_device(&self) -> bool {
        ((&self.0[0] >> 6) & 0x1) == 1
    }

    pub fn block_size(&self) -> usize {
        &self.0[0] & 0x3f
    }
}

pub struct TCBCap(RawCapability);

pub struct CNodeCap(RawCapability);

pub struct EndPointCap(RawCapability);

pub struct NotificationCap(RawCapability);

impl Capability for UntypedCap {
    fn init(addr: KernelVAddress, user_size: usize) -> Self {
        let mut raw_capability = RawCapability([0; 2]);

        todo!();
        Self(raw_capability)
    }
    fn from_raw(raw_cap: RawCapability) -> KernelResult<Self> {
        if let CapabilityType::Untyped = raw_cap.get_cap_type()? {
            Ok(Self(raw_cap))
        } else {
            Err(Err::UnexpectedCapType)
        }
    }

    fn get_object_size(user_size: usize) -> usize {
        todo!()
    }

    fn get_raw_cap(&self) -> RawCapability {
        self.0
    }
}
