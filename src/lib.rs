//! This crate contains a practical implementation of the x86 xsave semantics.
//!
//! We do not intend to support all possible variations of the instructures,
//! nor do we intend to calculate the size of the xsave area dynamically.
//! Instead, our practical strategy will overallocate the size of the xsave
//! area so that we get a constant size for the struct. This allows for
//! substantially easier embedding in other contexts.
//!
//! For example, clearing the extended CPU state is a simple:
//!
//! ```rust
//! use xsave::XSave;
//!
//! XSave::default().load();
//! ```
//!
//! Likewise, you can save and restore the extended CPU state like this:
//!
//! ```rust
//! use xsave::XSave;
//!
//! let mut xsave = XSave::default();
//! xsave.save();
//! xsave.load();
//! ```

#![cfg_attr(feature = "asm", feature(asm))]
#![deny(clippy::all)]
#![no_std]

use bitflags::bitflags;
use const_default::ConstDefault;

/// An MMX register
#[repr(C)]
#[derive(Copy, Clone, Default, Debug, ConstDefault)]
pub struct Mm([u8; 10]);

/// An MMX register field
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, ConstDefault)]
pub struct MmField {
    pub mm: Mm,
    reserved: [u8; 6],
}

/// An XMM register
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, ConstDefault)]
pub struct Xmm([u8; 16]);

bitflags! {
    /// The x87 Floating Point Control Word
    #[repr(transparent)]
    pub struct Fcw: u16 {
        const INVALID_OPERATION = 1 << 0;
        const DENORMALIZED_OPERAND = 1 << 1;
        const DIVIDE_BY_ZERO = 1 << 2;
        const OVERFLOW = 1 << 3;
        const UNDERFLOW = 1 << 4;
        const PRECISION = 1 << 5;
        const RESERVED6 = 1 << 6;
        const RESERVED7 = 1 << 7;
        const PRECISION_CONTROL0 = 1 << 8;
        const PRECISION_CONTROL1 = 1 << 9;
        const ROUNDING_CONTROL0 = 1 << 10;
        const ROUNDING_CONTROL1 = 1 << 11;
        const INFINITY_CONTROL = 1 << 12;
    }
}

impl ConstDefault for Fcw {
    const DEFAULT: Self = Self::from_bits_truncate(
        // NOTE: Section 8.1.5 declares that the default value of the FCW is
        // 0x37F. This includes the RESERVED6 bit for unknown reasons.
        Fcw::INVALID_OPERATION.bits
            | Fcw::DENORMALIZED_OPERAND.bits
            | Fcw::DIVIDE_BY_ZERO.bits
            | Fcw::OVERFLOW.bits
            | Fcw::UNDERFLOW.bits
            | Fcw::PRECISION.bits
            | Fcw::RESERVED6.bits
            | Fcw::PRECISION_CONTROL0.bits
            | Fcw::PRECISION_CONTROL1.bits,
    );
}

impl Default for Fcw {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

bitflags! {
    /// The x87 Floating Point Unit (FPU) Status Word
    ///
    /// For more details, see the Intel Developer Manual, Section 8.1.5
    #[repr(transparent)]
    #[derive(Default, ConstDefault)]
    pub struct Fsw: u16 {
        const INVALID_OPERATION = 1 << 0;
        const DENORMALIZED_OPERAND = 1 << 1;
        const DIVIDE_BY_ZERO = 1 << 2;
        const OVERFLOW = 1 << 3;
        const UNDERFLOW = 1 << 4;
        const PRECISION = 1 << 5;
        const STACK_FAULT = 1 << 6;
        const EXCEPTION_SUMMARY = 1 << 7;
        const CONDITION0 = 1 << 8;
        const CONDITION1 = 1 << 9;
        const CONDITION2 = 1 << 10;
        const CONDITION3 = 1 << 14;
        const FPU_BUSY = 1 << 15;
    }
}

bitflags! {
    /// The MXCSR register
    #[repr(transparent)]
    pub struct MxCsr: u32 {
        const INVALID_OPERATION = 1 << 0;
        const DENORMAL = 1 << 1;
        const DIVIDE_BY_ZERO = 1 << 2;
        const OVERFLOW = 1 << 3;
        const UNDERFLOW = 1 << 4;
        const PRECISION = 1 << 5;
        const DENORMALS_ARE_ZEROS = 1 << 6;

        const INVALID_OPERATION_MASK = 1 << 7;
        const DENORMAL_MASK = 1 << 8;
        const DIVIDE_BY_ZERO_MASK = 1 << 9;
        const OVERFLOW_MASK = 1 << 10;
        const UNDERFLOW_MASK = 1 << 11;
        const PRECISION_MASK = 1 << 12;

        const ROUNDING_CONTROL0 = 1 << 13;
        const ROUNDING_CONTROL1 = 1 << 14;
        const FLUSH_TO_ZERO = 1 << 15;
    }
}

impl ConstDefault for MxCsr {
    const DEFAULT: Self = Self::from_bits_truncate(
        MxCsr::INVALID_OPERATION_MASK.bits
            | MxCsr::DENORMAL_MASK.bits
            | MxCsr::DIVIDE_BY_ZERO_MASK.bits
            | MxCsr::OVERFLOW_MASK.bits
            | MxCsr::UNDERFLOW_MASK.bits
            | MxCsr::PRECISION_MASK.bits,
    );
}

impl Default for MxCsr {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

bitflags! {
    /// XCOMP_BV flags
    #[repr(transparent)]
    #[derive(Default, ConstDefault)]
    pub struct XCompBv: u64 {
        const COMPACT = 1 << 63;
    }
}

bitflags! {
    /// XSTATE_BV flags
    #[repr(transparent)]
    #[derive(Default, ConstDefault)]
    pub struct XStateBv: u64 {
        const X87 = 1 << 0;
        const SSE = 1 << 1;
        const AVX = 1 << 2;
        const BNDREGS = 1 << 3;
        const BNDCSR  = 1 << 4;
        const AVX512_OPMASK = 1 << 5;
        const AVX512_ZMM_HI256 = 1 << 6;
        const AVX512_HI16_ZMM = 1 << 7;
        const PT = 1 << 8;
        const PKRU = 1 << 9;
    }
}

/// The XSave Legacy Area
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct XSaveLegacy {
    pub fcw: Fcw,
    pub fsw: Fsw,
    pub ftw: u8,
    reserved0: u8,
    pub fop: u16,
    pub fip: u64,
    pub fdp: u64,
    pub mxcsr: MxCsr,
    pub mxcsr_mask: MxCsr,
    pub mm: [MmField; 8],
    pub xmm: [Xmm; 16],
    reserved1: [u64; 11],
    reserved2: [u8; 7],
}

impl ConstDefault for XSaveLegacy {
    const DEFAULT: Self = Self {
        fcw: Fcw::DEFAULT,
        fsw: Fsw::DEFAULT,
        ftw: 0,
        reserved0: 0,
        fop: 0,
        fip: 0,
        fdp: 0,
        mxcsr: MxCsr::DEFAULT,
        mxcsr_mask: MxCsr::from_bits_truncate(!0),
        mm: [MmField::DEFAULT; 8],
        xmm: [Xmm::DEFAULT; 16],
        reserved1: [0; 11],
        reserved2: [0; 7],
    };
}

/// The XSave Header Area
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, ConstDefault)]
pub struct XSaveHeader {
    pub xstate_bv: XStateBv,
    pub xcomp_bv: XCompBv,
    reserved: [u64; 6],
}

/// The XSave Extended Area
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, ConstDefault)]
struct XSaveExtend {
    reserved0: [u64; 24],
    reserved1: [[u64; 32]; 9],
}

/// An XSave buffer
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug, Default)]
pub struct XSave {
    pub legacy: XSaveLegacy,
    pub header: XSaveHeader,
    extend: XSaveExtend,
}

impl ConstDefault for XSave {
    const DEFAULT: Self = Self {
        legacy: XSaveLegacy::DEFAULT,
        header: XSaveHeader::DEFAULT,
        extend: XSaveExtend::DEFAULT,
    };
}

impl XSave {
    /// Save the extended CPU state
    #[inline(never)]
    #[cfg(feature = "asm")]
    pub extern "C" fn save(&mut self) {
        unsafe {
            asm!(
                "mov     eax, ~0",
                "mov     edx, ~0",
                "xsave   [{}]",

                in(reg) self,
                out("eax") _,
                out("edx") _,
            )
        }
    }

    /// Load the extended CPU state
    #[inline(never)]
    #[cfg(feature = "asm")]
    pub extern "C" fn load(&self) {
        unsafe {
            asm!(
                "mov     eax, ~0",
                "mov     edx, ~0",
                "xrstor  [{}]",

                in(reg) self,
                out("eax") _,
                out("edx") _,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::{align_of, size_of};

    #[test]
    fn default() {
        assert_eq!(XSave::DEFAULT.legacy.mxcsr.bits, 0x1F80);
        assert_eq!(XSave::DEFAULT.legacy.fcw.bits, 0x037F);
        assert_eq!(XSave::DEFAULT.legacy.fsw.bits, 0);
    }

    #[test]
    fn size() {
        assert_eq!(size_of::<XSaveLegacy>(), 512);
        assert_eq!(size_of::<XSaveHeader>(), 64);
        assert_eq!(size_of::<XSaveExtend>(), 2496);
        assert_eq!(size_of::<XSave>(), 3072);
    }

    #[test]
    fn align() {
        assert_eq!(align_of::<XSave>(), 64);
    }

    #[test]
    #[cfg(feature = "asm")]
    #[cfg(target_feature = "sse")]
    fn asm() {
        let mut xsave = XSave::default();

        let xmm0: f32 = 0.7;
        unsafe { asm!("", in("xmm0") xmm0) };

        xsave.save();

        let xmm0: f32;
        unsafe { asm!("", out("xmm0") xmm0) };
        assert_eq!(xmm0, 0.7);

        XSave::DEFAULT.load();

        let xmm0: f32;
        unsafe { asm!("", out("xmm0") xmm0) };
        assert_eq!(xmm0, 0.0);

        xsave.load();

        let xmm0: f32;
        unsafe { asm!("", out("xmm0") xmm0) };
        assert_eq!(xmm0, 0.7);
    }
}
