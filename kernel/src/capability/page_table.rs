use crate::address::PAGE_SIZE;
use crate::common::ErrKind;
use crate::kerr;
use crate::object::page_table::Page;
use crate::object::page_table::PageTable;
use crate::object::KObject;
use crate::riscv::sfence;
use crate::{
    address::{KernelVAddress, VirtAddr},
    capability::{Capability, CapabilityData, CapabilityType},
    common::KernelResult,
};

use super::Something;

/*
 * PageTable
 * RawCapability[0]
 * | padding 14 | is_root 1 | is_mapped 1 | mapped_address 48 |
 * 64                                            0
 */

impl KObject for PageTable {}

pub type PageTableCap = CapabilityData<PageTable>;

impl PageTableCap {
    pub fn map(&mut self, root_table: &mut Self, vaddr: VirtAddr) -> KernelResult<usize> {
        (!self.is_mapped())
            .then_some(())
            .ok_or(kerr!(ErrKind::PageTableAlreadyMapped))?;
        let parent_table = root_table.get_pagetable();
        let table = self.get_pagetable();
        let level = table.map(parent_table, vaddr)?;
        self.set_mapped(vaddr);
        Ok(level)
    }

    pub fn get_pagetable(&mut self) -> &mut PageTable {
        let address = self.get_address();
        let ptr: *mut PageTable = KernelVAddress::from(address).into();
        unsafe { ptr.as_mut().unwrap() }
    }

    pub unsafe fn activate(&mut self) -> KernelResult<()> {
        // TODO: Check
        self.is_root()
            .then_some(())
            .ok_or(kerr!(ErrKind::NotRootPageTable))?;
        let page_table = self.get_pagetable();
        unsafe {
            page_table.activate();
        }
        Ok(())
    }

    fn set_mapped(&mut self, vaddr: VirtAddr) {
        self.cap_dep_val |=
            (0x1 << 48) | (<VirtAddr as Into<usize>>::into(vaddr) & 0xffffffffffff) as u64
    }

    pub fn make_as_root(&mut self) -> KernelResult<()> {
        (!self.is_mapped())
            .then_some(())
            .ok_or(kerr!(ErrKind::PageTableAlreadyMapped))?;
        let root_table = self.get_pagetable();
        root_table.copy_global_mapping();
        let addr = VirtAddr::from(root_table as *const PageTable);
        self.set_mapped(addr);
        self.set_root();
        Ok(())
    }

    fn is_mapped(&self) -> bool {
        ((self.cap_dep_val >> 48) & 0x1) == 1
    }

    pub fn is_root(&self) -> bool {
        ((self.cap_dep_val >> 49) & 0x1) == 1
    }

    pub fn set_root(&mut self) {
        self.cap_dep_val |= 0x1 << 49
    }
}

impl Default for PageTable {
    fn default() -> Self {
        Self::new()
    }
}
/*
 * Page
 * RawCapability[0]
 * | padding 11 | right 3 | is_device 1 | is_mapped 1 | mapped_address 48 |
 * 64                                                                    0
 */

impl KObject for Page {}

pub type PageCap = CapabilityData<Page>;

impl PageCap {
    pub fn map(
        &mut self,
        root_table: &mut PageTableCap,
        vaddr: VirtAddr,
        flags: usize,
    ) -> KernelResult<()> {
        (!self.is_mapped())
            .then_some(())
            .ok_or(kerr!(ErrKind::PageAlreadyMapped))?;
        let parent_table = root_table.get_pagetable();
        let page = self.get_page();
        page.map(parent_table, vaddr, flags)?;
        self.set_mapped(vaddr);
        sfence();
        // TODO:
        // sfence_vma_vaddr()
        Ok(())
    }

    pub fn unmap(&mut self, root_table: &mut PageTableCap) -> KernelResult<()> {
        self.is_mapped()
            .then_some(())
            .ok_or(kerr!(ErrKind::PageNotMappedYet))?;
        let vaddr = self.get_mapped_address();
        let page = self.get_page();
        let root_table = root_table.get_pagetable();
        page.unmap(root_table, vaddr)?;
        self.set_unmapped();
        Ok(())
    }

    pub fn get_page(&mut self) -> &mut Page {
        let address = self.get_address();
        let ptr: *mut Page = KernelVAddress::from(address).into();
        unsafe { ptr.as_mut().unwrap() }
    }

    fn set_mapped(&mut self, vaddr: VirtAddr) {
        self.cap_dep_val |=
            (0x1 << 48) | (<VirtAddr as Into<usize>>::into(vaddr) & 0xffffffffffff) as u64
    }

    fn set_unmapped(&mut self) {
        // clear flag, right and mapped address
        self.cap_dep_val = 0
    }

    fn is_mapped(&self) -> bool {
        ((self.cap_dep_val >> 48) & 0x1) == 1
    }

    pub fn get_address_virtual(&self) -> KernelVAddress {
        self.get_address().into()
    }

    pub fn get_mapped_address(&self) -> VirtAddr {
        ((self.cap_dep_val & !(0xffff << 48)) as usize).into()
    }
}

impl Capability for PageTableCap {
    const CAP_TYPE: CapabilityType = CapabilityType::PageTable;
    type KernelObject = PageTable;

    fn init_object(&mut self) {
        let addr = KernelVAddress::from(self.get_address());
        let ptr = <KernelVAddress as Into<*mut Self::KernelObject>>::into(addr);
        unsafe {
            *ptr = PageTable::new();
        }
    }
    fn get_object_size<'a>(_user_size: usize) -> usize {
        PAGE_SIZE // page size, bytes
    }
    fn derive(&self, _src_slot: &crate::object::CNodeEntry<Something>) -> KernelResult<Self> {
        // self.is_mapped()
        //     .then_some(())
        //     .ok_or(kerr!(ErrKind::PageTableNotMappedYet))?;
        Ok(self.replicate())
    }
}

impl Capability for PageCap {
    const CAP_TYPE: CapabilityType = CapabilityType::Page;
    type KernelObject = Page;

    fn init_object(&mut self) {
        let addr = KernelVAddress::from(self.get_address());
        let ptr = <KernelVAddress as Into<*mut Self::KernelObject>>::into(addr);
        unsafe {
            *ptr = Page::new();
        }
    }

    fn get_object_size<'a>(_user_size: usize) -> usize {
        PAGE_SIZE // page size, bytes
    }

    fn derive(&self, _src_slot: &crate::object::CNodeEntry<Something>) -> KernelResult<Self> {
        self.is_mapped()
            .then_some(())
            .ok_or(kerr!(ErrKind::PageTableNotMappedYet))?;

        let mut cap = self.replicate();
        cap.cap_dep_val = 0;
        Ok(cap)
    }
}
