use super::{Capability, CapabilityType, RawCapability};
use crate::address::KernelVAddress;
use crate::common::KernelResult;
use crate::object::{Endpoint, ThreadControlBlock};

pub struct EndPointCap(RawCapability);

/*
 * RawCapability[1]
 *  | cap_type | can recieve | can reply | padding | address or none |
 * 64    5           1            1          9            48        0
 * RawCapability[0]
 * |                             badge                              |
 */
impl Capability for EndPointCap {
    const CAP_TYPE: CapabilityType = CapabilityType::EndPoint;
    // TODO
    type KernelObject = Endpoint;
    fn new(raw_cap: RawCapability) -> Self {
        Self(raw_cap)
    }
    fn get_raw_cap(&self) -> RawCapability {
        self.0
    }

    fn init_object<'x>(&mut self) {
        let addr = KernelVAddress::from(self.0.get_address());
        let ptr = <KernelVAddress as Into<*mut Self::KernelObject>>::into(addr);
        unsafe {
            *ptr = Self::KernelObject::new();
        }
    }
}

impl EndPointCap {
    /// return should be resche (because of blocking)
    pub fn send(&mut self, tcb: &mut ThreadControlBlock) -> bool {
        self.get_ep().send(tcb)
    }

    pub fn recv(&mut self, tcb: &mut ThreadControlBlock) -> bool {
        self.get_ep().recv(tcb)
    }

    fn get_ep(&mut self) -> &mut Endpoint {
        let addr = KernelVAddress::from(self.0.get_address());
        let ptr =
            <KernelVAddress as Into<*mut <EndPointCap as Capability>::KernelObject>>::into(addr);
        unsafe { &mut *ptr }
    }

    pub fn get_batch(&self) -> u64 {
        self.0.cap_dep_val
    }

    pub fn set_badge(&mut self, _val: u64) -> KernelResult<()> {
        todo!()
    }
}
