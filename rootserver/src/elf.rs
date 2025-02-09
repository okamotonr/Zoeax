use core::{cmp::min, ptr};
use libzoea::{
    caps::{CNodeCapability, Page, PageCapability, PageFlags, PageTable, PageTableCapability, UntypedCapability}, shared::{
        align_up,
        elf::{def::ProgramFlags, ProgramMapper},
        PAGE_SIZE,
    }, syscall::SysCallFailed, ErrKind
};

// TODO: Consider we have to borrow everything.
pub struct ElfProgramMapper<'a> {
    cnode: CNodeCapability,
    ut: UntypedCapability,
    root_table: &'a mut PageTableCapability,
    target_root_vspace: PageTableCapability,
    free_address: usize,
}


// TODO: DRY in kernel::init::root_server
// ElfMapper gets FnMut(map_page), Fn(flag analyzer)
// FnMut(
// usize or VAddr,
// usize or Paddr,
// F,
// &mut usize,
// usize,
// ) -> Result where C: Capability or some trait to enable
// Fn (u32) -> F: F is flag ((bool, bool, bool) or usize)
//
impl ProgramMapper for ElfProgramMapper<'_> {
    type Flag = PageFlags;
    type Error = SysCallFailed;

    fn get_flags(flag: u32) -> Self::Flag {
        let is_executable = ProgramFlags::is_executable(flag);
        let is_writable = ProgramFlags::is_writable(flag);
        let is_readable = ProgramFlags::is_readable(flag);
        PageFlags { is_writable, is_readable, is_executable }
    }

    fn map_program(
            &mut self,
            vaddr: usize,
            p_start_addr: *const u8,
            p_mem_size: usize,
            p_file_size: usize,
            flags: Self::Flag,
        ) -> Result<(), Self::Error> {
        let mut file_sz_rem = p_file_size;
        let page_num = (align_up(p_mem_size, PAGE_SIZE)) / PAGE_SIZE;
        for page_idx in 0..page_num {
            let offset = PAGE_SIZE * page_idx;
            let vaddr_n = vaddr + offset;
            self.map_page(
                vaddr_n,
                p_start_addr,
                flags,
                &mut file_sz_rem,
                offset,
            )?;
        }
        Ok(())

        
    }
}

impl<'a> ElfProgramMapper<'a> {
    pub fn new(
        cnode: CNodeCapability,
        ut: UntypedCapability,
        root_table: &'a mut PageTableCapability,
        target_root_vspace: PageTableCapability,
        free_address: usize,
    ) -> Self {
        Self {
            cnode,
            ut,
            root_table,
            target_root_vspace,
            free_address,
        }
    }

    pub fn try_new(
        mut cnode: CNodeCapability,
        mut ut: UntypedCapability,
        root_table: &'a mut PageTableCapability,
        free_address: usize,
    ) -> Result<Self, SysCallFailed> {

        let mut table_slot = cnode.get_slot()?;
        let target_root_vspace = ut
            .retype_single_with_fixed_size::<PageTable>(&mut table_slot)?;
        Ok(
            Self {
                cnode,
                ut,
                root_table,
                target_root_vspace,
                free_address
            }
        )
    }

    pub fn finalize(self) -> (CNodeCapability, UntypedCapability, PageTableCapability) {
        let cnode = self.cnode;
        let ut = self.ut;
        let target_root_vspace = self.target_root_vspace;
        (cnode, ut, target_root_vspace)
    }

    fn map_page(
        &mut self,
        vaddr: usize,
        p_start_addr: *const u8,
        flags: PageFlags,
        file_sz_rem: &mut usize,
        offset: usize,
    ) -> Result<(), SysCallFailed> {
        let page_cap = self.map_page_with_tables(vaddr, flags)?;
        if *file_sz_rem != 0 {
            let mut tmp_page_cap = self.cnode.copy(&page_cap)?;
            let copy_size = min(PAGE_SIZE, *file_sz_rem);
            tmp_page_cap.map(self.root_table, self.free_address, flags)?;
            unsafe {
                let copy_src = p_start_addr.add(offset);
                ptr::copy::<u8>(copy_src, self.free_address as *mut u8, copy_size);
            }
            *file_sz_rem = (*file_sz_rem).saturating_sub(PAGE_SIZE);
            tmp_page_cap.unmap(self.root_table)?;
            self.cnode.delete(tmp_page_cap)?;
        }
        Ok(())
    }

    fn map_page_with_tables(
        &mut self,
        vaddr: usize,
        flags: PageFlags,
    ) -> Result<PageCapability, SysCallFailed> {
        let mut slot = self.cnode.get_slot()?;
        let mut page_cap = self.ut.retype_single_with_fixed_size::<Page>(&mut slot)?;
        if let Err(e) = page_cap.map(&mut self.target_root_vspace, vaddr, flags) {
            match e {
                (ErrKind::PageTableNotMappedYet, value) => {
                    self.map_page_tables(vaddr, value)?;
                    page_cap.map(&mut self.target_root_vspace, vaddr, flags)?;
                }
                _ => Err((ErrKind::InvalidOperation, 0))?,
            }
        }
        Ok(page_cap)
    }

    fn map_page_tables(
        &mut self,
        vaddr: usize,
        _value: u16,
    ) -> Result<(), SysCallFailed> {
        loop {
            let mut table_slot = self.cnode.get_slot()?;
            let mut page_table_cap = self
                .ut
                .retype_single_with_fixed_size::<PageTable>(&mut table_slot)?;
            if let Ok(level) = page_table_cap.map(&mut self.target_root_vspace, vaddr) {
                if level == 0 {
                    break;
                }
            }
        }
        Ok(())
    }
}

