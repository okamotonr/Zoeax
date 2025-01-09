use core::marker::PhantomData;
use core::mem::align_of_val;

use crate::capability::Err;
use crate::capability::PhysAddr;
use crate::capability::{Capability, CapabilityType, RawCapability};
use crate::common::KernelResult;
use crate::object::CNode;
use crate::object::CNodeEntry;
use crate::object::Untyped;
use crate::vm::KernelVAddress;

use super::cnode::CNodeCap;
use super::endpoint::EndPointCap;
use super::notification::NotificationCap;
use super::tcb::TCBCap;

/*
 * RawCapability[0]
 * | 48 bit free_idx | 9 bit padd | 1 bit is_device | 6 bit block_size |
 * 64                                                                   0
 */

pub struct UntypedCap(RawCapability);

impl Capability for UntypedCap {
    const CAP_TYPE: CapabilityType = CapabilityType::Untyped;
    type KernelObject<'x> = Untyped;

    fn new(raw_cap: RawCapability) -> Self {
        Self(raw_cap)
    }
    fn create_cap_dep_val(addr: KernelVAddress, user_size: usize) -> usize {
        let is_device: usize = 0x00;
        let val: usize =
            (<KernelVAddress as Into<usize>>::into(addr) << 16) & is_device & user_size;

        val
    }

    fn get_object_size(user_size: usize) -> usize {
        return user_size;
    }

    fn get_raw_cap(&self) -> RawCapability {
        self.0
    }

    fn can_be_retyped_from_device_memory() -> bool {
        true
    }
    fn init_object(&mut self) -> () {
        ()
    }
}

const ADDRESS_LENGTH: usize = 48; // sv48

impl UntypedCap {
    pub fn retype<T: Capability>(
        &mut self,
        user_size: usize,
        num: usize,
    ) -> KernelResult<CapGenerator<T>> {
        // 1, can convert from device memory
        let is_device = self.is_device();
        if is_device {
            Self::can_be_retyped_from_device_memory()
                .then_some(())
                .ok_or(Err::CanNotNewFromDeviceMemory)?
        }
        let block_size = self.block_size();
        let object_size = num * T::get_object_size(user_size);

        // 2, whether memory is enough or not
        let new_block_size = block_size.checked_sub(object_size).ok_or(Err::NoMemory)?;

        let free_idx = self.get_free_index();
        // 3, create given type capabilities
        let cap_generator = CapGenerator::<T>::new(num, free_idx, object_size);
        let new_free_address = cap_generator.end_address;
        // 4, update self information
        let v = Self::create_cap_dep_val(new_free_address, new_block_size);
        self.0[0] = v;
        if is_device {
            self.mark_is_device()
        }
        Ok(cap_generator)
    }

    pub fn decode_invocation(
        inv_label: usize,
        length: usize,
        src_slot: &mut CNodeEntry,
        dest_cnode_cap: &mut CNodeCap,
        dest_offset: usize,
        user_size: usize,
        num: usize,
        new_type: CapabilityType,
    ) -> KernelResult<()> {
        let dest_cnode = dest_cnode_cap.get_writable(num, dest_offset)?;
        let mut untyped_cap = UntypedCap::try_from_raw(src_slot.cap())?;
        match new_type {
            CapabilityType::TCB => untyped_cap._invocation::<TCBCap>(
                length,
                src_slot,
                dest_cnode,
                user_size,
                num,
            ),
            CapabilityType::CNode => untyped_cap._invocation::<CNodeCap>(
                length,
                src_slot,
                dest_cnode,
                user_size,
                num,
            ),
            CapabilityType::EndPoint => untyped_cap._invocation::<EndPointCap>(
                length,
                src_slot,
                dest_cnode,
                user_size,
                num,
            ),
            CapabilityType::Notification => untyped_cap._invocation::<NotificationCap>(
                length,
                src_slot,
                dest_cnode,
                user_size,
                num,
            ),
            CapabilityType::PageTable => {
                todo!();
                untyped_cap._invocation::<EndPointCap>(
                    length,
                    src_slot,
                    dest_cnode,
                    user_size,
                    num,
                )
            },
            CapabilityType::Page => {
                todo!();
                untyped_cap._invocation::<EndPointCap>(
                    length,
                    src_slot,
                    dest_cnode,
                    user_size,
                    num,
                )
            }
            _ => Err(Err::UnknownCapType),
        }
    }

    fn _invocation<T: Capability>(
        &mut self,
        length: usize,
        src_slot: &mut CNodeEntry,
        dest_cnode: &mut CNode,
        user_size: usize,
        num: usize,
    ) -> KernelResult<()> {
        let cap_gen = self.retype::<T>(user_size, num)?.into_iter();
        for (i, mut cap) in cap_gen.enumerate() {
            cap.init_object();
            dest_cnode.insert_cap(src_slot, cap.get_raw_cap(), i);
        }
        src_slot.set_cap(self.get_raw_cap());
        Ok(())
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

    pub fn mark_is_device(&mut self) {
        self.0[1] = self.0[1] & 0x3f
    }
}

pub struct CapGenerator<C: Capability> {
    num: usize,              // mutable
    address: KernelVAddress, // mutable
    obj_size: usize,
    end_address: KernelVAddress,
    _phantom: PhantomData<fn() -> C>,
}

impl<C: Capability> CapGenerator<C> {
    pub fn new(num: usize, start_address: KernelVAddress, obj_size: usize) -> Self {
        let end_address = KernelVAddress::new(
            <KernelVAddress as Into<usize>>::into(start_address) + obj_size * num,
        );
        Self {
            num,
            address: start_address,
            obj_size,
            _phantom: PhantomData,
            end_address,
        }
    }
}

impl<C: Capability> Iterator for CapGenerator<C> {
    type Item = C;
    fn next(&mut self) -> Option<Self::Item> {
        if self.num == 0 {
            None
        } else {
            let cap = C::init(self.address, self.obj_size);
            self.address = self.address.add(self.obj_size);
            self.num -= 1;
            Some(cap)
        }
    }
}
