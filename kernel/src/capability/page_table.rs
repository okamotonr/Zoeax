use crate::common::Err;
use crate::object::page_table::Page;
use crate::object::page_table::PageTable;
use crate::{
    capability::{Capability, CapabilityType, RawCapability},
    common::KernelResult,
    memory::VirtAddr,
    vm::KernelVAddress,
};

/*
 * RawCapability[0]
 * | padding 15 | is_mapped 1 | mapped_address 48 |
 * 64                                            0
 */
pub struct PageTableCap(RawCapability);

impl PageTableCap {
    pub fn map(&mut self, parent: Self, vaddr: VirtAddr) -> KernelResult<()> {
        (!self.is_mapped())
            .then_some(())
            .ok_or(Err::PageTableAlreadyMapped)?;
        let parent_table = parent.get_pagetable();
        let table = self.get_pagetable();
        table.map(parent_table, vaddr)?;
        self.set_mapped(vaddr);
        Ok(())
    }

    pub fn get_pagetable(&self) -> &mut PageTable {
        let address = self.0.get_address();
        let ptr: *mut PageTable = KernelVAddress::from(address).into();
        unsafe { ptr.as_mut().unwrap() }
    }

    fn set_mapped(&mut self, vaddr: VirtAddr) {
        self.0[0] |= (0x1 << 48 | (<VirtAddr as Into<usize>>::into(vaddr) & 0xffffffffffff))
    }

    fn is_mapped(&self) -> bool {
        todo!()
    }
}

/*
 * RawCapability[0]
 * | padding 11 | right 3 | is_device 1 | is_mapped 1 | mapped_address 48 |
 * 64                                                                    0
 */
pub struct PageCap(RawCapability);

impl PageCap {
    pub fn map(
        &mut self,
        page_table: &mut PageTableCap,
        vaddr: VirtAddr,
        flags: usize,
    ) -> KernelResult<()> {
        (!self.is_mapped())
            .then_some(())
            .ok_or(Err::PageAlreadyMapped)?;
        let parent_table = page_table.get_pagetable();
        let page = self.get_page();
        page.map(parent_table, vaddr, flags)?;
        self.set_mapped(vaddr);
        Ok(())
    }

    pub fn get_page(&mut self) -> &mut Page {
        let address = self.0.get_address();
        let ptr: *mut Page = KernelVAddress::from(address).into();
        unsafe { ptr.as_mut().unwrap() }
    }

    fn set_mapped(&mut self, vaddr: VirtAddr) {
        self.0[0] |= (0x1 << 48 | (<VirtAddr as Into<usize>>::into(vaddr) & 0xffffffffffff))
    }

    fn is_mapped(&self) -> bool {
        todo!()
    }
}

impl Capability for PageTableCap {
    const CAP_TYPE: CapabilityType = CapabilityType::PageTable;
    type KernelObject<'x> = PageTable;
    fn get_raw_cap(&self) -> RawCapability {
        self.0
    }

    fn new(raw_cap: RawCapability) -> Self {
        Self(raw_cap)
    }

    fn init_object(&mut self) -> () {
        todo!()
    }
}

impl Capability for PageCap {
    const CAP_TYPE: CapabilityType = CapabilityType::Page;
    type KernelObject<'x> = Page;
    fn get_raw_cap(&self) -> RawCapability {
        self.0
    }

    fn new(raw_cap: RawCapability) -> Self {
        Self(raw_cap)
    }

    fn init_object(&mut self) -> () {
        todo!()
    }
}
