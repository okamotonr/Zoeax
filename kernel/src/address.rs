use crate::common::KernelResult;
use core::arch::naked_asm;

extern "C" {
    pub static __free_ram: u8;
    pub static __free_ram_end: u8;
}

pub const PAGE_SIZE: usize = 4096;
pub const KERNEL_V_ADDR_PFX: usize = 0xffff800000000000;

#[allow(dead_code)]
pub unsafe fn copy_to_user<T: Sized>(src: VirtAddr, dst: VirtAddr) -> KernelResult<()> {
    // TODO:
    //if dst < PAGE_SIZE.into() || dst > kernel_base.into() {
    //    return Err(Err::InvalidUserAddress)
    //}
    //if dst + size_of::<T>().into() > kernel_base.into() {
    //    return Err(Err::InvalidUserAddress)
    //}
    mem_copy_to_user(dst.addr as *mut T, src.addr as *const T, size_of::<T>());
    Ok(())
}

#[allow(dead_code)]
pub unsafe fn copy_from_user<T: Sized>(addr: VirtAddr, dst: *mut T) -> KernelResult<()> {
    // TODO:
    // if addr < PAGE_SIZE.into() || addr > kernel_base.into() {
    //     return Err(Err::InvalidUserAddress)
    // }
    // if addr + size_of::<T>().into() > kernel_base.into() {
    //     return Err(Err::InvalidUserAddress)
    // }
    mem_copy_from_user(dst, addr.addr as *const T, size_of::<T>());
    Ok(())
}

#[naked]
extern "C" fn mem_copy_from_user<T>(dst: *mut T, src: *const T, len: usize) {
    unsafe {
        naked_asm!(
            "beqz a2, 2f",
            "1:",
            "lb a3, 0(a1)",
            "sb a3, 0(a0)",
            "addi a1, a1, 1",
            "addi a0, a0, 1",
            "addi a2, a2, -1",
            "bnez a2, 1b",
            "2:",
            "ret"
        )
    }
}

#[naked]
extern "C" fn mem_copy_to_user<T>(dst: *mut T, src: *const T, len: usize) {
    unsafe {
        naked_asm!(
            //".global riscv32_usercopy2",
            "beqz a2, 2f", // a2 (コピー長) がゼロならラベル「2」にジャンプ
            "1:",
            "lb a3, 0(a1)", // a1レジスタの指すアドレス (カーネルポインタ) から1バイト読み込む
            //"riscv32_usercopy2:",
            "sb a3, 0(a0)", // a0レジスタの指すアドレス (ユーザーポインタ) に1バイト書き込む
            "addi a0, a0, 1", // a0レジスタ (ユーザーポインタ) を1バイト進める
            "addi a1, a1, 1", // a1レジスタ (カーネルポインタ) を1バイト進める
            "addi a2, a2, -1", // a2レジスタの値を1減らす
            "bnez a2, 1b",  // a2レジスタの値がゼロでなければラベル「1」にジャンプ
            "2:",
            "ret"
        )
    }
}
mod inner {
    use core::{fmt, marker::PhantomData, ops};
    pub trait AddressType {}
    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Address<T: AddressType> {
        pub addr: usize,
        _phantom: PhantomData<fn() -> T>,
    }

    impl<T: AddressType> Address<T> {
        pub const fn new(addr: usize) -> Self {
            Self {
                addr,
                _phantom: PhantomData,
            }
        }

        pub fn add(self, val: usize) -> Self {
            Self::new(self.addr + val)
        }

        pub fn bit_or(self, val: usize) -> Self {
            Self::new(self.addr | val)
        }

        pub fn bit_and(self, val: usize) -> Self {
            Self::new(self.addr & val)
        }
    }

    impl<T: AddressType> From<usize> for Address<T> {
        fn from(item: usize) -> Self {
            Self::new(item)
        }
    }

    impl<T: AddressType> From<Address<T>> for usize {
        fn from(item: Address<T>) -> Self {
            item.addr
        }
    }

    impl<T: AddressType, S> From<*const S> for Address<T> {
        fn from(value: *const S) -> Self {
            Self::new(value as usize)
        }
    }

    impl<T: AddressType, S> From<*mut S> for Address<T> {
        fn from(value: *mut S) -> Self {
            Self::new(value as usize)
        }
    }

    impl<T: AddressType, S> From<Address<T>> for *const S {
        fn from(value: Address<T>) -> Self {
            value.addr as *const S
        }
    }

    impl<T: AddressType, S> From<Address<T>> for *mut S {
        fn from(value: Address<T>) -> Self {
            value.addr as *mut S
        }
    }

    impl<T: AddressType> fmt::Debug for Address<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "Address {{0x{:x}, {}}}:",
                self.addr,
                core::any::type_name::<T>()
            )
        }
    }

    impl<T: AddressType> ops::Add for Address<T> {
        type Output = Self;
        fn add(self, rhs: Self) -> Self::Output {
            Self {
                addr: self.addr + rhs.addr,
                _phantom: PhantomData,
            }
        }
    }

    impl<T: AddressType> ops::AddAssign for Address<T> {
        fn add_assign(&mut self, rhs: Self) {
            self.addr += rhs.addr;
        }
    }

    impl<T: AddressType> ops::Sub for Address<T> {
        type Output = Self;
        fn sub(self, rhs: Self) -> Self::Output {
            Self::new(self.addr - rhs.addr)
        }
    }

    impl<T: AddressType> ops::SubAssign for Address<T> {
        fn sub_assign(&mut self, rhs: Self) {
            self.addr -= rhs.addr
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Physical;

    impl AddressType for Physical {}
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Virtual;
    impl AddressType for Virtual {}
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct KVirtual;
    impl AddressType for KVirtual {}
}

pub type PhysAddr = inner::Address<inner::Physical>;
pub type VirtAddr = inner::Address<inner::Virtual>;
pub type KernelVAddress = inner::Address<inner::KVirtual>;

impl From<PhysAddr> for VirtAddr {
    fn from(value: PhysAddr) -> Self {
        Self::new(value.addr)
    }
}

impl From<PhysAddr> for KernelVAddress {
    fn from(value: PhysAddr) -> Self {
        Self::new(value.addr | KERNEL_V_ADDR_PFX)
    }
}

impl From<KernelVAddress> for PhysAddr {
    fn from(value: KernelVAddress) -> Self {
        PhysAddr::new(value.addr & !KERNEL_V_ADDR_PFX)
    }
}

impl From<VirtAddr> for PhysAddr {
    fn from(value: VirtAddr) -> Self {
        PhysAddr::new(value.addr)
    }
}

impl From<KernelVAddress> for VirtAddr {
    fn from(value: KernelVAddress) -> Self {
        VirtAddr::new(value.addr)
    }
}

impl VirtAddr {
    #[inline]
    pub fn get_vpn(&self, idx: usize) -> usize {
        (self.addr >> (12 + idx * 9)) & 0x1ff
    }
}
