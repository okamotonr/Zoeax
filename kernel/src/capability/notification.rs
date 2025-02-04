use super::{Capability, CapabilityData, CapabilityType};
use crate::address::KernelVAddress;
use crate::common::KernelResult;
use crate::object::{KObject, Notification, ThreadControlBlock};

impl KObject for Notification {}

pub type NotificationCap = CapabilityData<Notification>;

/*
 * RawCapability[1]
 *  | cap_type | can recieve | can reply | padding | address or none |
 * 64    5           1            1          9            48        0
 * RawCapability[0]
 * |                             badge                              |
 */

impl Capability for NotificationCap {
    const CAP_TYPE: CapabilityType = CapabilityType::Notification;
    type KernelObject = Notification;

    fn create_cap_dep_val(_addr: KernelVAddress, _user_size: usize) -> usize {
        0 // badge
    }

    fn init_object(&mut self) {
        let addr = KernelVAddress::from(self.get_address());
        let ptr = <KernelVAddress as Into<*mut Self::KernelObject>>::into(addr);
        unsafe {
            *ptr = Self::KernelObject::new();
        }
    }
}

impl NotificationCap {
    pub fn send(&mut self) {
        let val = self.get_batch();
        self.get_notify().send_signal(val)
    }

    /// return should be resche (because of blocking)
    pub fn wait(&mut self, tcb: &mut ThreadControlBlock) -> bool {
        self.get_notify().wait_signal(tcb)
    }

    fn get_notify(&mut self) -> &mut Notification {
        let addr = KernelVAddress::from(self.get_address());
        let ptr =
            <KernelVAddress as Into<*mut <NotificationCap as Capability>::KernelObject>>::into(
                addr,
            );
        unsafe { &mut *ptr }
    }

    fn get_batch(&self) -> u64 {
        self.cap_dep_val
    }

    #[allow(dead_code)]
    pub fn set_badge(&mut self, _val: u64) -> KernelResult<()> {
        todo!()
    }
}
