use super::{RawCapability, CapabilityType, Capability};
use crate::object::Notification;
use crate::vm::KernelVAddress;

pub struct NotificationCap(RawCapability);

impl Capability for NotificationCap {
    const CAP_TYPE: CapabilityType = CapabilityType::Notification;
    // TODO
    type KernelObject = Notification;
    fn new(raw_cap: RawCapability) -> Self {
        Self(raw_cap)
    }
    fn get_raw_cap(&self) -> RawCapability {
        self.0
    }

    fn init_object(&mut self) {
        let addr = KernelVAddress::from(self.0.get_address());
        let ptr = <KernelVAddress as Into<*mut Self::KernelObject>>::into(addr); 
        unsafe {
            *ptr = Self::KernelObject::new();
        }
    }

}


