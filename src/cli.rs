/*
MIT License

Copyright (c) 2024 Philipp Schuster

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

use clap::{Parser, Subcommand, ValueEnum};
use std::str::FromStr;

/// A virtual address in hexadecimal representation. It be provided to the CLI
/// as `0x123` or `0x1234_5678`. The `0x` prefix is required. It must be within
/// the range of `u64`. Can be truncated to `u32`. In this case, the upper 32
/// bits are discarded.
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Hash, derive_more::Display)]
#[display("0x{_0:016x}")]
pub struct VirtualAddress(u64);

impl VirtualAddress {
    const PREFIX: &'static str = "0x";

    /// Returns whether this address is canonical for the given x86_64 virtual
    /// address width.
    ///
    /// An x86_64 virtual address is canonical when every bit above the most
    /// significant implemented address bit is a copy of that bit. Therefore,
    /// 4-level paging requires bits 63 through 48 to equal bit 47, while
    /// 5-level paging requires bits 63 through 57 to equal bit 56.
    fn is_canonical(self, address_bits: u32) -> bool {
        debug_assert!((1..64).contains(&address_bits));

        let unimplemented_bits = 64 - address_bits;
        (((self.0 << unimplemented_bits) as i64) >> unimplemented_bits) as u64 == self.0
    }
}

/// Describes errors that happened when users tries to input a [`VirtualAddress`]
/// via the CLI.
#[derive(Copy, Clone, Debug, thiserror::Error, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub enum VirtualAddressError {
    /// The virtual address must begin with the prefix 0x.
    #[error("virtual address must begin with the prefix 0x")]
    MissingPrefix,
    /// The virtual address could not be parsed as number of type `u64`.
    #[error("virtual address could not be parsed as number of type `u64`")]
    ParseIntError,
}

/// Describes errors found after all CLI arguments have been parsed.
#[derive(Copy, Clone, Debug, thiserror::Error, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub enum CliValidationError {
    /// The virtual address is not canonical for the selected x86_64 paging
    /// mode.
    #[error("{virtual_address} is not a canonical {address_bits}-bit x86_64 virtual address")]
    NonCanonicalVirtualAddress {
        /// The invalid virtual address.
        virtual_address: VirtualAddress,
        /// Number of virtual address bits implemented by the paging mode.
        address_bits: u32,
    },
}

impl From<u64> for VirtualAddress {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<VirtualAddress> for u64 {
    fn from(value: VirtualAddress) -> Self {
        value.0
    }
}

impl From<VirtualAddress> for u32 {
    fn from(value: VirtualAddress) -> Self {
        (value.0 & 0xffffffff) as Self
    }
}

impl FromStr for VirtualAddress {
    type Err = VirtualAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Remove underscores and other clutter which are allowed for the input.
        let s = s.trim().to_lowercase().replace('_', "");

        if !s.starts_with(Self::PREFIX) {
            return Err(VirtualAddressError::MissingPrefix);
        }

        // string without the prefix
        let s_without_prefix = &s.as_str()[Self::PREFIX.len()..];

        u64::from_str_radix(s_without_prefix, 16)
            .map(Self)
            .map_err(|e| {
                eprintln!("{e}");
                VirtualAddressError::ParseIntError
            })
    }
}

/// CLI args definition of this application for `clap`.
#[derive(Parser)]
#[command(author, version, about)]
pub struct CliArgs {
    #[arg()]
    /// A virtual address in hexadecimal representation. It be provided to
    /// the CLI as `0x123` or `0x1234_5678`. The `0x` prefix is required.
    /// It must be within the range of `u64`.
    pub virtual_address: VirtualAddress,

    /// Architecture/Paging implementation.
    #[command(subcommand)]
    pub architecture: Architecture,

    #[arg(long, value_enum)]
    pub color: Option<ColorOption>,
}

impl CliArgs {
    /// Validates relationships between parsed CLI arguments.
    ///
    /// In particular, x86_64 addresses must be canonical for the selected
    /// paging mode. See [`VirtualAddress::is_canonical`] for the definition of
    /// a canonical address.
    pub fn validate(&self) -> Result<(), CliValidationError> {
        let Some(address_bits) = self.architecture.virtual_address_bits() else {
            return Ok(());
        };

        if self.virtual_address.is_canonical(address_bits) {
            Ok(())
        } else {
            Err(CliValidationError::NonCanonicalVirtualAddress {
                virtual_address: self.virtual_address,
                address_bits,
            })
        }
    }
}

/// Whether colors and other ANSI escape sequences should be used.
#[derive(Copy, Clone, Debug, Default, PartialOrd, PartialEq, Ord, Eq, Hash, ValueEnum)]
pub enum ColorOption {
    /// Never use ANSI escape sequences.
    Never,
    /// Use ANSI escape sequences if stdout points to a TTY, i.e., is not
    /// redirected.
    #[default]
    Auto,
    /// Always use ANSI escape sequences.
    Always,
}

/// Supported architectures with options. Each architecture is a subcommand of
/// the CLI.
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Hash, Subcommand)]
pub enum Architecture {
    /// Calculate page table index information for x86. x86 uses a 2-level page
    /// table.
    X86 {
        /// Physical Page Extension.
        #[arg(long, default_value = "false")]
        pae: bool,
    },
    /// Calculate page table index information for x86_64. x86_64 uses a 4-level
    /// whose structure is similar to x86 with Page Address Extension (PAE) but
    /// with 64-bit virtual addresses.
    #[command(id = "x86_64")]
    X86_64 {
        /// Optional feature of x86_64 that adds one additional level to the
        /// 4-level page-table of
        /// `x86_64`.
        #[arg(short = '5', long, default_value = "false")]
        five_level: bool,
    },
}

impl Architecture {
    /// Returns the implemented virtual address width when canonical address
    /// validation applies.
    const fn virtual_address_bits(self) -> Option<u32> {
        match self {
            Self::X86 { .. } => None,
            Self::X86_64 { five_level: false } => Some(48),
            Self::X86_64 { five_level: true } => Some(57),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_addr_from_str() {
        assert_eq!(VirtualAddress::from_str("0x123"), Ok(0x123.into()));
        assert_eq!(
            VirtualAddress::from_str("0xdead_beef"),
            Ok(0xdead_beef.into())
        );
        assert_eq!(
            VirtualAddress::from_str("    0xdEAd_bEEF    "),
            Ok(0xdead_beef.into())
        );
    }

    #[test]
    fn test_virtual_addr_64_to_32_bit() {
        let v_addr = VirtualAddress::from_str("0xdead_beef_1337_1337");
        assert_eq!(v_addr, Ok(0xdead_beef_1337_1337.into()));
        let v_addr = v_addr.unwrap();
        assert_eq!(u32::from(v_addr), 0x1337_1337);
    }

    #[test]
    fn test_validate_x86_64_four_level_canonical_addresses() {
        let mut args = CliArgs {
            virtual_address: 0x0000_7fff_ffff_ffff.into(),
            architecture: Architecture::X86_64 { five_level: false },
            color: None,
        };
        assert_eq!(args.validate(), Ok(()));

        args.virtual_address = 0xffff_8000_0000_0000.into();
        assert_eq!(args.validate(), Ok(()));

        args.virtual_address = 0x0000_8000_0000_0000.into();
        assert!(matches!(
            args.validate(),
            Err(CliValidationError::NonCanonicalVirtualAddress {
                address_bits: 48,
                ..
            })
        ));

        args.virtual_address = 0xffff_7fff_ffff_ffff.into();
        assert!(matches!(
            args.validate(),
            Err(CliValidationError::NonCanonicalVirtualAddress {
                address_bits: 48,
                ..
            })
        ));
    }

    #[test]
    fn test_validate_x86_64_five_level_canonical_addresses() {
        let mut args = CliArgs {
            virtual_address: 0x00ff_ffff_ffff_ffff.into(),
            architecture: Architecture::X86_64 { five_level: true },
            color: None,
        };
        assert_eq!(args.validate(), Ok(()));

        args.virtual_address = 0xff00_0000_0000_0000.into();
        assert_eq!(args.validate(), Ok(()));

        args.virtual_address = 0x0100_0000_0000_0000.into();
        assert!(matches!(
            args.validate(),
            Err(CliValidationError::NonCanonicalVirtualAddress {
                address_bits: 57,
                ..
            })
        ));

        args.virtual_address = 0xfeff_ffff_ffff_ffff.into();
        assert!(matches!(
            args.validate(),
            Err(CliValidationError::NonCanonicalVirtualAddress {
                address_bits: 57,
                ..
            })
        ));
    }

    #[test]
    fn test_validate_x86_accepts_addresses_that_are_truncated() {
        let args = CliArgs {
            virtual_address: u64::MAX.into(),
            architecture: Architecture::X86 { pae: false },
            color: None,
        };

        assert_eq!(args.validate(), Ok(()));
    }
}
