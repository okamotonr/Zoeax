use def::{Elf64Hdr, Elf64Phdr, PHeaders, ProgramType};

pub mod def;

pub trait ProgramMapper {
    type Flag;
    type Error;

    fn get_flags(flag: u32) -> Self::Flag;
    fn map_program(
        &mut self,
        vaddr: usize,
        p_start_addr: *const u8,
        p_mem_size: usize,
        p_file_size: usize,
        flags: Self::Flag,
    ) -> Result<(), Self::Error>;
}

impl Elf64Hdr {
    pub fn map_self<Mapper: ProgramMapper>(
        &self,
        mapper: &mut Mapper,
    ) -> Result<(), Mapper::Error> {
        for (p_header, p_start_addr) in PHeaders::new(self) {
            self.map_program(p_header, p_start_addr, mapper)?;
        }
        Ok(())
    }

    fn map_program<Mapper: ProgramMapper>(
        &self,
        p_header: &Elf64Phdr,
        p_start_addr: *const u8,
        mapper: &mut Mapper,
    ) -> Result<(), Mapper::Error> {
        if !(p_header.p_type == ProgramType::Load) {
            return Ok(());
        };
        let flags = Mapper::get_flags(p_header.p_flags);
        let p_memsz = p_header.p_memsz;
        let vaddr = p_header.p_vaddr;
        let p_filesz = p_header.p_filesz;
        mapper.map_program(vaddr, p_start_addr, p_memsz, p_filesz, flags)
    }
}
