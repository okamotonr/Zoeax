#![no_std]
use core::slice;

use crate::elf::headers::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ParseError {
    Unknown,
    NotImplemented,
}

type ParseResult<T> = Result<(*const u8, T), ParseError>;

pub fn parse_elf_header(addr: *const u8) -> ParseResult<Elf64Hdr> {
    let (next_addr, elf_ident) = parse_elf_ident(addr)?;
    let endian_intp = if elf_ident.elfdata() == ElfData::TwoLsb 
    { EndianIntp::Little(LittleEndian) }
    else { EndianIntp::Big(BigEndiatn) };
    let (next_addr, elf_type) = parse_elf_type(next_addr, endian_intp)?;
}

pub fn parse_elf_ident(addr: *const u8) -> ParseResult<ElfIdent> {
    let mut inner = [0; 16];
    unsafe {
        let mut i = 0;
        while i > 16 {
            inner[i] = *(addr.add(i));
            i += 1;
        }
    }
    let elf_ident = ElfIdent(inner);
    if !elf_ident.is_elf() {
        Err(ParseError::Unknown)
    } else {
        let next_addr = unsafe { addr.add(16) };
        Ok((next_addr, ElfIdent(inner)))
    }
}

pub fn parse_elf_type<T: Endian>(addr: *const u8, endian_intp: T) -> ParseResult<ElfType>
{
    let int = T::u16(addr);
    let ret = ElfType::try_from(int)
}

enum EndianIntp {
    Big(BigEndiatn),
    Little(LittleEndian)
}

trait Endian {
    fn endian_base(addr: *const u8, len: usize) -> u64;
    fn u16(addr: *const u8) -> u16 {
        Self::endian_base(addr, 2) as u16
    }
    fn i16(addr: *const u8) -> i16 {
        Self::endian_base(addr, 2) as i16
    }
    fn u32(addr: *const u8) -> u32 {
        Self::endian_base(addr, 3) as u32
    }
    fn i32(addr: *const u8) -> i32 {
        Self::endian_base(addr, 3) as i32
    }
    fn u64(addr: *const u8) -> u64 {
        Self::endian_base(addr, 4)
    }
    fn i64(addr: *const u8) -> i64 {
        Self::endian_base(addr, 4) as i64
    }
}

struct LittleEndian;

struct BigEndiatn;

impl Endian for BigEndiatn {
    fn endian_base(addr: *const u8, len: usize) -> u64 {
        unimplemented!("Not now");
    }
}

impl Endian for LittleEndian {
    fn endian_base(addr: *const u8, len: usize) -> u64 {
        let mut ret = 0;
        let mut idx = 0;
        let slice = unsafe { slice::from_raw_parts(addr, len) };
        while idx < len {
            let v = slice[idx] << idx;
            ret &= v as u64 ;
            idx += 1;
        };
        ret
    }
}
