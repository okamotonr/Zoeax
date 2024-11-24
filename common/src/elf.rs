const EI_DENT: usize = 16;

type Elf64Addr = usize;
type Elf64Off = usize;
type Elf64Word = u32;
type Elf64Xword = u64;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[repr(C)]
pub struct Elf64Hdr {
    pub e_ident: ElfIdent,
    pub e_type: ElfType,
    pub e_machine: ElfMachine,
    pub e_version: ElfVersion,
    pub e_entry: Elf64Addr,
    pub e_phoff: Elf64Off,
    pub e_shoff: Elf64Off,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

impl Elf64Hdr {
    pub fn get_sheader(&self, elf_header_addr: *const usize, idx: u16) -> Option<*const Elf64Shdr> {
        if self.e_shnum <= idx {
            None
        } else {
            unsafe {
                let base_addr = elf_header_addr.add(self.e_shoff).cast::<Elf64Shdr>();
                Some(base_addr.add(idx as usize))
            }
        }
    }

    pub fn get_pheader(&self, elf_header_addr: *const usize, idx: u16) -> Option<*const Elf64Phdr> {
        if self.e_phnum <= idx {
            None
        } else {
            unsafe {
                let base_addr = elf_header_addr.byte_add(self.e_phoff) as *const Elf64Phdr;
                Some(base_addr.add(idx as usize))
            }
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[repr(C)]
pub struct ElfIdent(pub [u8; EI_DENT]);

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[repr(u8)]
pub enum ElfClass {
    None,
    Class32,
    Class64,
    ClassNum
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[repr(u8)]
pub enum ElfData {
    None,
    TwoLsb,
    TwoMsb
}

impl ElfIdent {
    pub fn is_elf(&self) -> bool {
        self.0[0] == 0x7f
            && self.0[1] == b'E'
            && self.0[2] == b'L'
            && self.0[3] == b'F'
    }

    pub fn elfclass(&self) -> ElfClass {
        match self.0[4] {
            0 => ElfClass::None,
            1 => ElfClass::Class32,
            2 => ElfClass::Class64,
            _ => panic!("Unknown")
        }
        
    }

    pub fn elfdata(&self) -> ElfData {
        match self.0[5] {
            0 => ElfData::None,
            1 => ElfData::TwoLsb,
            2 => ElfData::TwoMsb,
            _ => panic!("Unknown")
        }
    }

    pub fn elfversion(&self) -> ElfVersion {
        match self.0[6] {
            0 => ElfVersion::None,
            1 => ElfVersion::Current,
            _ => panic!("Unknown")
        }
    }
}

#[repr(u16)]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ElfType {
    None,
    Rel,
    Exec,
    Dyn,
    Core
}

// TODO: riscv
#[repr(u16)]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ElfMachine {
    None,
    M32,
    Sparc,
    I386,
    M68k,
    M88k,
    I860,
    Mips,
    Parisc,
    Sparc32Plus,
    PPC,
    S390,
    Arm,
    Sh,
    Sparcv9,
    IA64,
    X86_64,
    Vax
}

#[repr(u32)]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ElfVersion {
    None,
    Current
}

// Program header
#[repr(C)]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Elf64Phdr {
    pub p_type: ProgramType,
    pub p_flags: u32,
    pub p_offset: Elf64Off,
    pub p_vaddr: Elf64Addr,
    pub p_paddr: Elf64Addr,
    pub p_filesz: usize,
    pub p_memsz: usize,
    pub p_align: usize,
}

#[repr(u32)]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ProgramType {
    Null,
    Load,
    Dynamic,
    Interp,
    Note,
    Shlib,
    Phdr,
    Tls,
} 

#[repr(u32)]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ProgramFlags {
    X = 0x01,
    W = 0x02,
    R = 0x04
}

impl ProgramFlags {
    #[inline]
    pub fn is_executable(flags: u32) -> bool {
        (Self::X as u32 & flags) == Self::X as u32
    }
    pub fn is_writable(flags: u32) -> bool {
        (Self::W as u32 & flags) == Self::W as u32
    }
    pub fn is_readable(flags: u32) -> bool {
        (Self::R as u32 & flags) == Self::R as u32
    }
}

// Section header
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Elf64Shdr {
    pub sh_name: u32,
    pub sh_type: SectionType,
    pub sh_flags: usize,
    pub sh_addr: Elf64Addr,
    pub sh_offset: Elf64Off,
    pub sh_size: usize,
    pub sh_link: u32,
    pub sh_info: u32,
    pub sh_addralign: usize,
    pub sh_entsize: usize,
}

#[repr(u32)]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum SectionType {
    Null,
    Progbits,
    Symtab,
    Strtab,
    Rela,
    Hash,
    Dynamic,
    Note,
    Notebits,
    Rel,
    Shlib,
    Dynsym,
    Loproc,
    Hyproc,
    Louser,
    Hiuser,
}

// String and symbol table
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Elf64Sym {
    pub st_name: u32,
    pub st_info: StInfo,
    pub st_other: u8,
    pub st_shndx: u16,
    pub st_value: Elf64Addr,
    pub st_size: usize,
}

#[repr(u8)]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum StInfo {
    Notype,
    Object,
    Func,
    Section,
    File,
    TLoproc,
    THiproc,
    Local,
    Global,
    Weak,
    BLoproc,
    BHiproc
}

#[repr(u8)]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum StOther {
    Default,
    Internal,
    Hidden,
    Protected
}

impl StOther {
    pub fn visibility(&self) {
        unimplemented!();
    }
}

// Relocation entries (Rel & Rela)
pub struct Elf64Rel {
    pub r_offset: Elf64Addr,
    pub r_info: usize,
}

pub struct Elf64Rela {
    pub r_offset: Elf64Addr,
    pub r_info: usize,
    pub r_addend: isize,
}

// Dynamic tags (Dyn)
pub struct Elf64Dyn {
    pub d_tag: Dtag,
    pub d_un: DUnion,
}

#[repr(i64)]
pub enum Dtag {
    Null,
    Needed,
    Pltrelsz,
    Pltgot,
    Hash,
    Strtab,
    Symtab,
    Rela,
    Relasz,
    Relaent,
    Strsz,
    Syment,
    Init,
    Fini,
    Soname,
    Rpath,
    Symbolic,
    Rel,
    Relsz,
    Relent,
    Pltrel,
    Debug,
    Textrel,
    Jmprel,
    BindNow,
    Runpath,
    Loproc,
    Hiproc
}

pub union DUnion {
    pub d_val: Elf64Xword,
    pub d_ptr: Elf64Addr,
}

// Notes (Nhdr)
pub struct Elf64Nhdr {
    pub n_namesz: Elf64Word,
    pub n_descz: Elf64Word,
    pub n_type: Elf64Word,
}
