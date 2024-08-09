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
use std::error::Error;
use std::fmt;
use std::str::FromStr;

/// A virtual address in hexadecimal representation. It be provided to the CLI
/// as `0x123` or `0x1234_5678`. The `0x` prefix is required. It must be within
/// the range of `u64`. Can be truncated to `u32`. In this case, the upper 32
/// bits are discarded.
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct VirtualAddress(u64);

impl fmt::Display for VirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:016x}", self.0)
    }
}

impl VirtualAddress {
    const PREFIX: &'static str = "0x";
}

/// Describes errors that happened when users tries to input a [`VirtualAddress`]
/// via the CLI.
#[derive(Copy, Clone, Debug, derive_more::Display, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub enum VirtualAddressError {
    /// The virtual address must begin with the prefix 0x.
    #[display("The virtual address must begin with the prefix 0x.")]
    MissingPrefix,
    /// The virtual address could not be parsed as number as `u64`
    #[display("The virtual address could not be parsed as number as `u64`.")]
    ParseIntError,
}

impl Error for VirtualAddressError {}

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
}
