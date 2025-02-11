use super::{Capability, CapabilityData, CapabilityType};
use crate::address::KernelVAddress;
use crate::common::KernelResult;
use crate::object::{Endpoint, KObject, ThreadControlBlock};

impl KObject for Endpoint {}

pub type EndPointCap = CapabilityData<Endpoint>;

/*
 * RawCapability[1]
 *  | cap_type | can recieve | can reply | padding | address or none |
 * 64    5           1            1          9            48        0
 * RawCapability[0]
 * |                             badge                              |
 */
impl Capability for EndPointCap {
    const CAP_TYPE: CapabilityType = CapabilityType::EndPoint;
    type KernelObject = Endpoint;

    fn init_object<'x>(&mut self) {
        let addr = KernelVAddress::from(self.get_address());
        let ptr = <KernelVAddress as Into<*mut Self::KernelObject>>::into(addr);
        unsafe {
            *ptr = Self::KernelObject::new();
        }
    }

    fn derive(
        &self,
        _src_slot: &crate::object::CNodeEntry<super::Something>,
    ) -> KernelResult<Self> {
        let mut ret = self.replicate();
        ret.cap_dep_val = 0;
        Ok(ret)
    }
}

impl EndPointCap {
    /// return should be resche (because of blocking)
    pub fn send(&mut self, tcb: &mut ThreadControlBlock) -> bool {
        tcb.badge = self.get_badge() as usize;
        self.get_ep().send(tcb)
    }

    pub fn recv(&mut self, tcb: &mut ThreadControlBlock) -> bool {
        self.get_ep().recv(tcb)
    }

    fn get_ep(&mut self) -> &mut Endpoint {
        let addr = KernelVAddress::from(self.get_address());
        let ptr =
            <KernelVAddress as Into<*mut <EndPointCap as Capability>::KernelObject>>::into(addr);
        unsafe { &mut *ptr }
    }

    pub fn get_badge(&self) -> u64 {
        self.cap_dep_val
    }

    #[allow(dead_code)]
    pub fn set_badge(&mut self, _val: u64) -> KernelResult<()> {
        todo!()
    }
}
