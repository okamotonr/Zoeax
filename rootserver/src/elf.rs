use core::{cmp::min, ptr};
use libzoea::{
    caps::{CNodeCapability, Page, PageCapability, PageFlags, PageTable, PageTableCapability, TCBCapability, UntypedCapability}, shared::{
        align_up,
        elf::def::{Elf64Hdr, Elf64Phdr, PHeaders, ProgramFlags, ProgramType},
        PAGE_SIZE,
    }, syscall::SysCallFailed, ErrKind
};

pub struct ElfMapper<'a> {
    cnode_mgr: CNodeCapability,
    ut_mgr: UntypedCapability,
    root_table: &'a mut PageTableCapability,
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
impl<'a> ElfMapper<'a> {
    pub fn new(
        cnode_mgr: CNodeCapability,
        ut_mgr: UntypedCapability,
        root_table: &'a mut PageTableCapability,
        free_address: usize,
    ) -> Self {
        Self {
            cnode_mgr,
            ut_mgr,
            root_table,
            free_address,
        }
    }

    pub fn map_elf(&mut self, elf_image: &Elf64Hdr, tcb_cap: &mut TCBCapability) -> Result<(), SysCallFailed> {
        /*
         * 1, create root vspace,
         * 2, mapping image,
         * 3, set root vspace into tcb
         * 4, write eip
         */
        let mut vroot_slot = self.cnode_mgr.get_slot()?;
        let mut vroot = self
            .ut_mgr
            .retype_single_with_fixed_size::<PageTable>(&mut vroot_slot)?;
        for (p_header, p_start_addr) in PHeaders::new(elf_image) {
            self.map_program(p_header, p_start_addr, &mut vroot)?
        }
        let entry = elf_image.e_entry;
        Ok(())
    }

    fn map_program(
        &mut self,
        p_header: &Elf64Phdr,
        p_start_addr: *const u8,
        target_root_space: &mut PageTableCapability,
    ) -> Result<(), SysCallFailed> {
        if !(p_header.p_type == ProgramType::Load) {
            return Ok(());
        }
        let flags = get_flags(p_header.p_flags);
        let page_num = (align_up(p_header.p_memsz, PAGE_SIZE)) / PAGE_SIZE;
        let mut file_sz_rem = p_header.p_filesz;
        let vaddr = p_header.p_vaddr;
        let mut file_sz_rem = p_header.p_filesz;
        for page_idx in 0..page_num {
            let offset = PAGE_SIZE * page_idx;
            let vaddr_n = vaddr + offset;
            self.map_page(
                vaddr_n,
                p_start_addr,
                flags,
                &mut file_sz_rem,
                offset,
                target_root_space,
            )?;
        }
        Ok(())
    }

    fn map_page(
        &mut self,
        vaddr: usize,
        p_start_addr: *const u8,
        flags: PageFlags,
        file_sz_rem: &mut usize,
        offset: usize,
        target_root_space: &mut PageTableCapability,
    ) -> Result<(), SysCallFailed> {
        let page_cap = self.map_page_with_tables(target_root_space, vaddr, flags)?;
        if *file_sz_rem != 0 {
            let copy_src = p_start_addr.add(offset);
            let tmp_page_cap = self.cnode_mgr.copy(page_cap)?;
            let copy_size = min(PAGE_SIZE, *file_sz_rem);
            tmp_page_cap.map(self.root_table, self.free_address)?;
            unsafe {
                ptr::copy::<u8>(copy_src, self.free_address as *mut u8, copy_size);
            }
            *file_sz_rem = (*file_sz_rem).saturating_sub(PAGE_SIZE);
            tmp_page_cap.ummap(self.root_table)?;
            self.cnode_mgr.delete(tmp_page_cap)?;
        }
        Ok(())
    }

    fn map_page_with_tables(
        &mut self,
        target_root_space: &mut PageTableCapability,
        vaddr: usize,
        flags: PageFlags,
    ) -> Result<PageCapability, SysCallFailed> {
        let mut slot = self.cnode_mgr.get_slot()?;
        let mut page_cap = self.ut_mgr.retype_single_with_fixed_size::<Page>(&mut slot)?;
        if let Err(e) = page_cap.map(target_root_space, vaddr, flags) {
            match e {
                (ErrKind::PageTableNotMappedYet, value) => {
                    self.map_page_tables(target_root_space, vaddr, value)?;
                    page_cap.map(vaddr, target_root_space, flags)?;
                }
                _ => Err((ErrKind::InvalidOperation, 0))?,
            }
        }
        Ok(page_cap)
    }

    fn map_page_tables(
        &mut self,
        target_root_space: &mut PageTableCapability,
        vaddr: usize,
        value: usize,
    ) -> Result<(), SysCallFailed> {
        loop {
            let mut table_slot = self.cnode_mgr.get_slot()?;
            let mut page_table_cap = self
                .ut_mgr
                .retype_single_with_fixed_size::<PageTable>(&mut table_slot)?;
            if let Ok(level) = page_table_cap.map(target_root_space, vaddr) {
                if level == 0 {
                    break;
                }
            }
        }
        Ok(())
    }
}

fn get_flags(flags: u32) -> PageFlags {
    let is_executable = ProgramFlags::is_executable(flags);
    let is_writable = ProgramFlags::is_writable(flags);
    let is_readable = ProgramFlags::is_readable(flags);
    PageFlags { is_writable, is_readable, is_executable }
}
