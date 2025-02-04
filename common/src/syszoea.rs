use crate::{syscall::untyped_retype, IPCBuffer};
use kernel::{CapabilityType, KernelResult};

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

pub struct TCBInformation {}

pub struct CNodeInformation {}

pub struct PageTableInformation {}

pub struct PageInformation {}

pub struct EndpointInformation {}

pub struct NotificationInformation {}

type UntypedCapability = Capability<UntypedData>;

impl UntypedCapability {
    pub fn retype_mul<T>(
        &mut self,
        dest_ptr: usize,
        dest_depth: u32,
        user_size: usize,
        buffer: &mut [Capability<T>],
        num: u32,
    ) -> KernelResult<()>
    where
        T: Cap,
    {
        // TODO: adapt multi number capabilities
        // TODO: use depth,
        untyped_retype(
            self.cap_ptr,
            self.cap_depth,
            dest_ptr,
            dest_depth,
            user_size,
            num,
            T::CAP_TYPE,
        )?;
        for i in 0..num {
            let new_c = T::default();
            buffer[i as usize] = Capability {
                cap_ptr: dest_ptr,
                cap_depth: self.cap_depth,
                cap_data: new_c,
            }
        }
        Ok(())
    }

    pub fn retype_single<T: Cap>(
        &mut self,
        dest_ptr: usize,
        dest_depth: u32,
        user_size: usize,
    ) -> KernelResult<Capability<T>> {
        let num = 1;
        untyped_retype(
            self.cap_ptr,
            self.cap_depth,
            dest_ptr,
            dest_depth,
            user_size,
            num,
            T::CAP_TYPE,
        )?;
        let new_c = T::default();
        Ok(Capability {
            cap_ptr: dest_ptr,
            cap_depth: self.cap_depth,
            cap_data: new_c,
        })
    }
}
