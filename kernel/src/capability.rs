use crate::{
    address::{KernelVAddress, PhysAddr, VirtAddr},
    common::{ErrKind, KernelResult},
    kerr,
    object::{CNodeEntry, KObject},
};

use core::{marker::PhantomData, num::NonZeroU8};
use core::mem;

pub mod cnode;
pub mod endpoint;
pub mod notification;
pub mod page_table;
pub mod tcb;
pub mod untyped;

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
// Or K trait bound is not required.
pub struct CapabilityData<K: KObject> {
    cap_type: NonZeroU8,
    cap_right: u8,
    address_top: u16,
    address_bottom: u32,
    cap_dep_val: u64,
    _obj_type: PhantomData<K>
}

/*
 * RawCapability[1]
 * | cap_type | cap_right | padding | address or none |
 * 64   5         5           6              48
 */

// This represents in slot capability.
pub struct Something;
impl KObject for Something {}

pub type CapInSlot = CapabilityData<Something>;

pub trait IntoSomething {
    fn into_something(&self) -> &CapabilityData<Something> {
        unsafe { 
            let ptr = self as *const Self as *const CapabilityData<Something>;
            ptr.as_ref().unwrap()
        }
    }

    fn into_something_mut(&mut self) -> &mut CapabilityData<Something> {
        unsafe {
            let ptr = self as *mut Self as *mut CapabilityData<Something>;
            ptr.as_mut().unwrap()
        }
    }
}

impl CapInSlot {
    pub fn as_capability<NK>(&mut self) -> KernelResult<&mut CapabilityData<NK>>
        where 
            NK: KObject,
            CapabilityData<NK>: Capability
    {
        let cap_type = self.get_cap_type()?;
        (CapabilityData::<NK>::CAP_TYPE == cap_type).then_some(()).ok_or(kerr!(ErrKind::UnexpectedCapType))?;
        unsafe {
            let ptr = self as *mut CapabilityData<Something> as *mut CapabilityData<NK>;
            Ok(ptr.as_mut().unwrap())
        }
    }
}

impl IntoSomething for CapInSlot {
    fn into_something(&self) -> &CapabilityData<Something> {
        self
    }

    fn into_something_mut(&mut self) -> &mut CapabilityData<Something> {
        self
    }
}

// Or don't have to implement cap trait.
impl Capability for CapInSlot {
    const CAP_TYPE: CapabilityType = CapabilityType::Anything;
    type KernelObject = Something;
    fn derive(&self, _src_slot: &CNodeEntry<Something>) -> KernelResult<Self> {
        todo!()
    }
    fn init_object(&mut self) {
        todo!()
    }
    fn get_object_size(_user_size: usize) -> usize {
        todo!()
    }
    fn create_cap_dep_val(_addr: KernelVAddress, _user_size: usize) -> usize {
        todo!()
    }
    fn can_be_retyped_from_device_memory() -> bool {
        todo!()
    }
}

impl<K: KObject> CapabilityData<K> 
where  CapabilityData<K>: Capability {

    pub fn init(address: KernelVAddress, user_size: usize) -> Self {
        let cap_type = Self::CAP_TYPE;
        let cap_dep_val = Self::create_cap_dep_val(address, user_size);
        Self::new(cap_type, address.into(), cap_dep_val as u64)
    }
    
    pub fn new(cap_type: CapabilityType, address: PhysAddr, cap_dep_val: u64) -> Self {
        let mut ret = Self {
            cap_type: NonZeroU8::new(cap_type as u8).unwrap(),
            cap_right: 0,
            address_top: 0,
            address_bottom: 0,
            cap_dep_val,
            _obj_type: PhantomData
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

    pub fn replicate(&self) -> Self {
        Self {
            cap_type: self.cap_type,
            cap_right: self.cap_right,
            address_top: self.address_top,
            address_bottom: self.address_bottom,
            cap_dep_val: self.cap_dep_val,
            _obj_type: self._obj_type
        }
    }
}
//
// impl<K: KObject> fmt::Debug for CapabilityData<K> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(
//             f,
//             "captype: {:?}, address: {:?}",
//             self.get_cap_type(),
//             self.get_address()
//         )
//     }
// }

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum CapabilityType {
    Anything = 11,
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
            val if val == Self::Untyped as u8 => Ok(Self::Untyped),
            val if val == Self::TCB as u8 => Ok(Self::TCB),
            val if val == Self::EndPoint as u8 => Ok(Self::EndPoint),
            val if val == Self::CNode as u8 => Ok(Self::CNode),
            val if val == Self::Notification as u8 => Ok(Self::Notification),
            2 => Ok(Self::PageTable),
            4 => Ok(Self::Page),
            _ => Err(kerr!(ErrKind::UnknownCapType)),
        }
    }
}

// TODO: Change of capability should change raw_cap in slot.
pub trait Capability// : IntoSomething 
where
    Self: Sized,
{
    const CAP_TYPE: CapabilityType;
    type KernelObject: KObject;

    fn create_cap_dep_val(_addr: KernelVAddress, _user_size: usize) -> usize {
        0
    }
    fn get_object_size(_user_size: usize) -> usize {
        mem::size_of::<Self::KernelObject>()
    }
    fn can_be_retyped_from_device_memory() -> bool {
        false
    }
    fn derive(&self, _src_slot: &CNodeEntry<Something>) -> KernelResult<Self> {
        Err(kerr!(ErrKind::CanNotDerivable))
    }
    fn init_object(&mut self);
}

pub fn up_cast<K: KObject>(cap: CapabilityData<K>) -> CapInSlot {
    CapInSlot {
        cap_type: cap.cap_type,
        cap_dep_val: cap.cap_dep_val,
        cap_right: cap.cap_right,
        address_bottom: cap.address_bottom,
        address_top: cap.address_top,
        _obj_type: PhantomData
    }
}
