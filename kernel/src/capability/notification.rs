use super::{RawCapability, CapabilityType, Capability};
use crate::object::Notification;
use crate::vm::KernelVAddress;

pub struct NotificationCap(RawCapability);

impl Capability for NotificationCap {
    const CAP_TYPE: CapabilityType = CapabilityType::Notification;
    // TODO
    type KernelObject<'x> = Notification<'static>;
    fn new(raw_cap: RawCapability) -> Self {
        Self(raw_cap)
    }
    fn get_raw_cap(&self) -> RawCapability {
        self.0
    }
}


