/*
MIT License

Copyright (c) 2022 Philipp Schuster

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

//! CLI utility that helps you to calculate indices into the page table from a virtual address. For
//! x86, it outputs the indices into the page tables for both, 32-bit and 64-bit paging.

#![deny(
    clippy::all,
    clippy::cargo,
    clippy::nursery,
    // clippy::restriction,
    // clippy::pedantic
)]
// now allow a few rules which are denied by the above statement
// --> they are ridiculous and not necessary
#![allow(
    clippy::suboptimal_flops,
    clippy::redundant_pub_crate,
    clippy::fallible_impl_from
)]
// I can't do anything about this; fault of the dependencies
#![allow(clippy::multiple_crate_versions)]
// allow: required because of derive_more::Display macro
#![allow(clippy::use_self)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(rustdoc::all)]

use clap::{Parser, ValueEnum};
use derive_more::Display;
use nu_ansi_term::{Color, Style};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// Creates a bitmask from a number that describes how many ones there should be (0..=64).
/// The ones are filled in from the right side.
fn num_to_bitmask(mut val: u64) -> u64 {
    assert!(val < 65, "The value must be between 0..=64. Is {}", val);
    let mut bitmask = 0;
    while val > 0 {
        bitmask <<= 1;
        bitmask |= 1;
        val -= 1;
    }
    bitmask
}

/// Calculates the index into the page table for the given level and the amount of
/// paging bits. It returns a tuple with
/// - the index into the page table (the index of the entry, not the byte offset!)
/// - the address where only the bits are set that are relevant for the provided level
/// - the amount how much the address was shifted, depending on the level
///
/// # Parameters
/// - `INDEX_BITS` - number of how many bits index into each page table (e.g. 10 on x86 or 9 on x86 with PAE or x86_64)
/// - `PAGE_OFFSET_BITS` - number of how many bits index into the page (e.g. 12 on x86)
fn calculate_page_table_index<const INDEX_BITS: u64, const PAGE_OFFSET_BITS: u64>(
    addr: u64,
    level: u64,
) -> (u64, u64, u64) {
    assert!(
        level > 0,
        "level must be 1 or more. level 0 means the page itself."
    );

    // number of bits to shift.
    let shift = INDEX_BITS * (level - 1) + PAGE_OFFSET_BITS;

    let shifted_addr = addr >> shift;

    let bitmask = num_to_bitmask(INDEX_BITS);

    let index = shifted_addr & bitmask;
    let relevant_part_of_addr = addr & (bitmask << shift);

    (index, relevant_part_of_addr, shift)
}

/// Supported architectures for the page table calculation.
#[derive(Copy, Clone, Debug, Default, PartialOrd, PartialEq, Ord, Eq, Hash, ValueEnum)]
pub enum Architecture {
    /// Page tables for x86 (with and without Physical Address Extension (PAE)) and x86_64.
    #[default]
    X86,
}

impl Architecture {

    fn print_paging_info(&self, addr: &VirtualAddress) {
        match self {
            Architecture::X86 => {
                const X86_ENTRY_SIZE: u64 = 4;
                const X86_64_ENTRY_SIZE: u64 = 8;
                const X86_ADDR_LEN: u64 = 32;
                const X86_64_ADDR_LEN: u64 = 64;

                // truncated to 32-bit for the 32-bit representation
                let bit32_address = addr.0 & 0xffffffff;

                // 32-bit mode
                let (l2_index, _l2_bits, l2_shift) =
                    calculate_page_table_index::<10, 12>(addr.0, 2);
                let (l1_index, _l1_bits, l1_shift) =
                    calculate_page_table_index::<10, 12>(addr.0, 1);

                println!("{}", Style::new().bold().paint("x86 32-bit paging"));
                println!("- Uses a 2-level page table and 10 bits to index into each table.");
                println!("- Each entry is 32-bit long. A page table has 1024 entries.");
                println!("- Huge pages have a size of 2^22 == 4 MiB.");
                println!();
                println!("address       : 0x{:x}", bit32_address);
                println!("address (bits): 0b{:032b}", bit32_address);
                print!("level 2 bits  : 0b");
                Self::print_relevant_bits::<10, 12, X86_ADDR_LEN>(l2_index, l2_shift, 2);
                println!();
                print!("level 1 bits  : 0b");
                Self::print_relevant_bits::<10, 12, X86_ADDR_LEN>(l1_index, l1_shift, 1);
                println!();
                println!("level 2 entry index : {l2_index:>6}  (number of entry)");
                println!("level 2 entry offset: 0x{:04x}  (offset into the page table for that entry)", l2_index * X86_ENTRY_SIZE);
                println!("level 1 entry index : {l1_index:>6}");
                println!("level 1 entry offset: 0x{:04x}", l1_index * X86_ENTRY_SIZE);

                println!();

                // 364-bit mode
                let (l4_index, _l4_bits, l4_shift) = calculate_page_table_index::<9, 12>(addr.0, 4);
                let (l3_index, _l3_bits, l3_shift) = calculate_page_table_index::<9, 12>(addr.0, 3);
                let (l2_index, _l2_bits, l2_shift) = calculate_page_table_index::<9, 12>(addr.0, 2);
                let (l1_index, _l1_bits, l1_shift) = calculate_page_table_index::<9, 12>(addr.0, 1);

                println!("{}", Style::new().bold().paint("x86 64-bit paging"));
                println!("- Uses a 4-level page table and 9 bits to index into each table.");
                println!("- Each entry is 64-bit long. A page table has 512 entries.");
                println!("- Huge pages have a size of 2^21 == 2 MiB or 2^30 == 1 GiB.");
                println!();
                println!("address       : {addr}");
                println!("address (bits): 0b{:064b}", addr.0);

                print!("level 4 bits  : 0b");
                Self::print_relevant_bits::<9, 12, X86_64_ADDR_LEN>(l4_index, l4_shift, 4);
                println!();

                print!("level 3 bits  : 0b");
                Self::print_relevant_bits::<9, 12, X86_64_ADDR_LEN>(l3_index, l3_shift, 3);
                println!();

                print!("level 2 bits  : 0b");
                Self::print_relevant_bits::<9, 12, X86_64_ADDR_LEN>(l2_index, l2_shift, 2);
                println!();

                print!("level 1 bits  : 0b");
                Self::print_relevant_bits::<9, 12, X86_64_ADDR_LEN>(l1_index, l1_shift, 1);
                println!();

                println!("level 4 entry index : {l4_index:>6}  (number of entry)");
                println!("level 4 entry offset: 0x{:04x}  (offset into the page table for that entry)", l4_index * X86_64_ENTRY_SIZE);
                println!("level 3 entry index : {l3_index:>6}");
                println!("level 3 entry offset: 0x{:04x}", l3_index * X86_64_ENTRY_SIZE);
                println!("level 2 entry index : {l2_index:>6}");
                println!("level 2 entry offset: 0x{:04x}", l2_index * X86_64_ENTRY_SIZE);
                println!("level 1 entry index : {l1_index:>6}");
                println!("level 1 entry offset: 0x{:04x}", l1_index * X86_64_ENTRY_SIZE);
            }
        }
    }

    // Prints the relevant bits used for the indexing and highlights them in red.
    fn print_relevant_bits<const INDEX_BITS: u64, const PAGE_OFFSET_BITS: u64, const ADDR_LEN: u64>(
        index: u64,
        shift: u64,
        level: u64,
    ) {
        assert!(level > 0);
        assert!(ADDR_LEN == 32 || ADDR_LEN == 64, "only 32-bit or 64-bit addresses are supported");

        use core::fmt::Write;

        let mut buf = String::new();

        write!(
            &mut buf,
            "{}",
            Style::new()
                .fg(Color::Red)
                .paint(format!("{index:0bits$b}", bits = INDEX_BITS as usize))
        )
        .unwrap();

        let zeroes_after = shift;
        write!(&mut buf, "{}", "0".repeat(zeroes_after as usize)).unwrap();

        // 64: the app prints each string as u64
        let zeroes_before = ADDR_LEN - zeroes_after - INDEX_BITS;

        print!("{}", "0".repeat(zeroes_before as usize));

        print!("{buf}");
    }
}

/// A virtual address in hexadecimal representation. It be provided to the CLI as `0x123` or
/// `0x1234_5678`. The `0x` prefix is required. It must be within the range of `u64`.
#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Hash)]
struct VirtualAddress(u64);

impl Display for VirtualAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:016x}", self.0)
    }
}

impl VirtualAddress {
    const PREFIX: &'static str = "0x";
}

/// Describes errors that happened when users tries to input a [`VirtualAddress`]
/// via the CLI.
#[derive(Copy, Clone, Debug, Display, PartialOrd, PartialEq, Ord, Eq, Hash)]
enum VirtualAddressError {
    /// The virtual address must beginn with the prefix 0x.
    #[display = "The virtual address must beginn with the prefix 0x."]
    MissingPrefix,
    /// The virtual address could not be parsed as number as `u64`
    #[display = "The virtual address could not be parsed as number as `u64`."]
    ParseIntError,
}

impl Error for VirtualAddressError {}

impl FromStr for VirtualAddress {
    type Err = VirtualAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // remove underscores which are allowed to
        let s = s.to_lowercase().replace('_', "");

        if !s.starts_with(VirtualAddress::PREFIX) {
            return Err(VirtualAddressError::MissingPrefix);
        }

        // string without the prefix
        let s_without_prefix = &s.as_str()[VirtualAddress::PREFIX.len()..];

        u64::from_str_radix(s_without_prefix, 16)
            .map(VirtualAddress)
            .map_err(|e| {
                eprintln!("{}", e);
                VirtualAddressError::ParseIntError
            })
    }
}

/// CLI args definition of this application for `clap`.
#[derive(Parser)]
#[command(author, version, about)]
struct CliArgs {
    /// A virtual address in hexadecimal representation. It be provided to the CLI as `0x123` or
    /// `0x1234_5678`. The `0x` prefix is required. It must be within the range of `u64`.
    virtual_address: VirtualAddress,

    #[arg(short, long, value_enum)]
    architecture: Option<Architecture>,
}

fn main() {
    // parse the CLI args. parse() is generated by clap.
    let cli: CliArgs = CliArgs::parse();

    // according to nu_ansi_doc, this is required
    #[cfg(target_os = "windows")]
    let _ = nu_ansi_term::enable_ansi_support();

    println!("{}", Style::new().bold().paint("Page Table Calculator"));
    println!();
    cli.architecture
        .unwrap_or_default()
        .print_paging_info(&cli.virtual_address)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_num_to_bitmask() {
        assert_eq!(num_to_bitmask(0), 0);
        assert_eq!(num_to_bitmask(1), 1);
        assert_eq!(num_to_bitmask(2), 0b11);
        assert_eq!(num_to_bitmask(4), 0xf);
        assert_eq!(num_to_bitmask(64), !0);
    }

    #[test]
    fn test_calculate_page_table_index_x86() {
        // a 32-bit address written so that it is seperated by the corresponding levels
        // of page table on x86.
        #[allow(clippy::unusual_byte_groupings)]
        let addr = 0b1111111111_1010101010_001111000011;

        let (l2_index, l2_bits, _) = calculate_page_table_index::<10, 12>(addr, 2);
        assert_eq!(
            l2_index, 0b1111111111,
            "Should be 0b000100000 but is {l2_index:#b}",
        );
        let expected_bits: u64 = 0b1111111111 << (10 + 12);
        assert_eq!(
            l2_bits, expected_bits,
            "Should be {l2_bits:#b} but is {expected_bits:#b}"
        );

        let (l1_index, l1_bits, _) = calculate_page_table_index::<10, 12>(addr, 1);
        assert_eq!(
            l1_index, 0b1010101010,
            "Should be 0b000100000 but is {l1_index:#b}",
        );
        let expected_bits: u64 = 0b1010101010 << 12;
        assert_eq!(
            l1_bits, expected_bits,
            "Should be {l1_bits:#b} but is {expected_bits:#b}"
        );
    }

    #[test]
    #[allow(clippy::identity_op)]
    fn test_calculate_page_table_index_64() {
        // a 32-bit address written so that it is separated by the corresponding levels
        // of page table on x86_64.
        #[allow(clippy::unusual_byte_groupings)]
        let addr = 0b000100000_000011111_111111111_010101010_001111000011;

        let (l4_index, l4_bits, _) = calculate_page_table_index::<9, 12>(addr, 4);
        assert_eq!(
            l4_index, 0b000100000,
            "Should be 0b000100000 but is {l4_index:#b}"
        );
        let expected_bits: u64 = 0b000100000 << (3 * 9 + 12);
        assert_eq!(
            l4_bits, expected_bits,
            "Should be {l4_bits:#b} but is {expected_bits:#b}"
        );

        let (l3_index, l3_bits, _) = calculate_page_table_index::<9, 12>(addr, 3);
        assert_eq!(
            l3_index, 0b000011111,
            "Should be 0b000011111 but is {l3_index:#b}"
        );
        let expected_bits: u64 = 0b000011111 << (2 * 9 + 12);
        assert_eq!(
            l3_bits, expected_bits,
            "Should be {l3_bits:#b} but is {expected_bits:#b}"
        );

        let (l2_index, l2_bits, _) = calculate_page_table_index::<9, 12>(addr, 2);
        assert_eq!(
            l2_index, 0b111111111,
            "Should be 0b111111111 but is {l2_index:#b}"
        );
        let expected_bits: u64 = 0b111111111 << (1 * 9 + 12);
        assert_eq!(
            l2_bits, expected_bits,
            "Should be {l2_bits:#b} but is {expected_bits:#b}"
        );

        let (l1_index, l1_bits, _) = calculate_page_table_index::<9, 12>(addr, 1);
        assert_eq!(
            l1_index, 0b010101010,
            "Should be 0b010101010 but is {l1_index:#b}"
        );
        let expected_bits: u64 = 0b010101010 << 12;
        assert_eq!(
            l1_bits, expected_bits,
            "Should be {l1_bits:#b} but is {expected_bits:#b}"
        );
    }
}
