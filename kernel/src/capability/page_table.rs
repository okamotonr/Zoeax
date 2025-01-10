use crate::common::Err;
use crate::memory::PAGE_SIZE;
use crate::object::page_table::Page;
use crate::object::page_table::PageTable;
use crate::print;
use crate::println;
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
    pub fn map(&mut self, root_table: &mut Self, vaddr: VirtAddr) -> KernelResult<usize> {
        (!self.is_mapped())
            .then_some(())
            .ok_or(Err::PageTableAlreadyMapped)?;
        let parent_table = root_table.get_pagetable();
        let table = self.get_pagetable();
        let level = table.map(parent_table, vaddr)?;
        self.set_mapped(vaddr);
        Ok(level)
    }

    pub fn get_pagetable(&self) -> &mut PageTable {
        let address = self.0.get_address();
        let ptr: *mut PageTable = KernelVAddress::from(address).into();
        unsafe { ptr.as_mut().unwrap() }
    }

    pub unsafe fn activate(&self) -> KernelResult<()> {
        self.is_mapped().then_some(()).ok_or(Err::PageTableNotMappedYet)?;
        let page_table = self.get_pagetable();
        println!("call activation");
        unsafe {
            Ok(page_table.activate())
        }
    }

    fn set_mapped(&mut self, vaddr: VirtAddr) {
        self.0[0] |= 0x1 << 48 | (<VirtAddr as Into<usize>>::into(vaddr) & 0xffffffffffff)
    }

    pub fn root_map(&mut self) -> KernelResult<()> {
        (!self.is_mapped())
            .then_some(())
            .ok_or(Err::PageTableAlreadyMapped)?;
        let vaddr = self.get_pagetable();
        let addr = VirtAddr::from(vaddr as *const PageTable);
        println!("{addr:?}");
        Ok(self.set_mapped(addr))
    }

    fn is_mapped(&self) -> bool {
        ((self.0[0] >> 48) & 0x1) == 1
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
        root_table: &mut PageTableCap,
        vaddr: VirtAddr,
        flags: usize,
    ) -> KernelResult<()> {
        (!self.is_mapped())
            .then_some(())
            .ok_or(Err::PageAlreadyMapped)?;
        let parent_table = root_table.get_pagetable();
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
        ((self.0[0] >> 48) & 0x1) == 1
    }
    pub fn get_address(&self) -> KernelVAddress {
        self.0.get_address().into()
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
    fn get_object_size<'a>(_user_size: usize) -> usize {
        PAGE_SIZE// page size, bytes
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

    fn get_object_size<'a>(_user_size: usize) -> usize {
        PAGE_SIZE // page size, bytes
    }
}
