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

//! Module with utilities to calculate the index into a page table for a given
//! page table and given paging characteristics.

use crate::addr_width::AddrWidth;
use crate::cli::VirtualAddress;

/// Contains the page table lookup meta info for a virtual address and a certain
/// level. Meta means that only information for the lookup itself are included
/// but not the lookup itself.
#[derive(Debug)]
pub struct PageTableLookupMetaInfo {
    /// Virtual address used to get the lookup info.
    #[allow(unused)]
    pub v_addr: VirtualAddress,
    /// Used level for the lookup.
    pub level: u64,
    /// Index into the page table. Between 0 and N-1, where N is the amount of
    /// entries for the page table.
    pub index: u64,
    /// Amount of bits needed for a shift of the virtual address so that the
    /// index bits stand on the most-right position.
    #[allow(unused)]
    pub shift: u64,
    /// Like `v_addr` but all bits irrelevant for the given level are zeroes.
    #[allow(unused)]
    pub relevant_part_of_addr: u64,
}

/// Calculates the index into the page table for the given level and the
/// given paging implementation characteristics.
///
/// # Parameters
/// - `index_bits` - number of how many bits index into each page table (e.g.
///   10 on x86 or 9 on x86 with PAE or `x86_64`)
/// - `page_offset_bits` - number of how many bits index into the page (e.g. 12
///   on `x86` and `x86_64`, i.e., 4096 bytes per page)
/// - `addr` - Virtual Address used to look-up the page table.
/// - `level` - Level of the page table. Must be bigger than zero!
/// - `addr_width` - Width of the address. See [`AddrWidth`].
pub fn calculate_page_table_index(
    index_bits: u64,
    page_offset_bits: u64,
    v_addr: impl Into<VirtualAddress>,
    // Level is always at least 1, as level 0 means the page itself is indexed.
    level: u64,
    addr_width: AddrWidth,
) -> PageTableLookupMetaInfo {
    assert!(index_bits > 0);
    assert!(page_offset_bits > 0);
    assert!(level > 0);

    let v_addr = v_addr.into();
    let addr = u64::from(v_addr);
    let addr = if addr_width == AddrWidth::Bits32 {
        addr & u32::MAX as u64
    } else {
        addr
    };

    // Shift the bits that index into the page table to the right.
    // To do that, we calc the number of bits to shift the virtual address.
    let shift = index_bits * (level - 1) + page_offset_bits;

    let shifted_addr = addr >> shift;

    let bitmask = bit_ops::bitops_u64::create_mask(index_bits);

    let index = shifted_addr & bitmask;
    let relevant_part_of_addr = addr & (bitmask << shift);

    PageTableLookupMetaInfo {
        v_addr,
        level,
        index,
        shift,
        relevant_part_of_addr,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_page_table_index_x86() {
        // a 32-bit address written so that it is separated by the corresponding levels
        // of page table on x86.
        #[allow(clippy::unusual_byte_groupings)]
        let addr = 0b1111111111_1010101010_001111000011;

        {
            let PageTableLookupMetaInfo {
                index: l2_index,
                relevant_part_of_addr: l2_bits,
                ..
            } = calculate_page_table_index(10, 12, addr, 2, AddrWidth::Bits32);
            assert_eq!(
                l2_index, 0b1111111111,
                "Should be 0b1111111111 but is {l2_index:#b}",
            );
            let expected_bits: u64 = 0b1111111111 << (10 + 12);
            assert_eq!(
                l2_bits, expected_bits,
                "Should be {l2_bits:#b} but is {expected_bits:#b}"
            );
        }

        {
            let PageTableLookupMetaInfo {
                index: l1_index,
                relevant_part_of_addr: l1_bits,
                ..
            } = calculate_page_table_index(10, 12, addr, 1, AddrWidth::Bits32);
            assert_eq!(
                l1_index, 0b1010101010,
                "Should be 0b1010101010 but is {l1_index:#b}",
            );
            let expected_bits: u64 = 0b1010101010 << 12;
            assert_eq!(
                l1_bits, expected_bits,
                "Should be {l1_bits:#b} but is {expected_bits:#b}"
            );
        }
    }

    #[test]
    fn test_calculate_page_table_index_x86_pae() {
        // a 32-bit address written so that it is separated by the corresponding
        // levels of page table on x86 with PAE.
        #[allow(clippy::unusual_byte_groupings)]
        let addr = 0b10_111111111_010101010_001111000011;

        {
            let PageTableLookupMetaInfo {
                index: l3_index,
                relevant_part_of_addr: l3_bits,
                ..
            } = calculate_page_table_index(9, 12, addr, 3, AddrWidth::Bits32);
            assert_eq!(l3_index, 0b10, "Should be 0b10 but is {l3_index:#b}",);
            let expected_bits: u64 = 0b10 << (9 * 2 + 12);
            assert_eq!(
                l3_bits, expected_bits,
                "Should be {l3_bits:#b} but is {expected_bits:#b}"
            );
        }

        {
            let PageTableLookupMetaInfo {
                index: l2_index,
                relevant_part_of_addr: l2_bits,
                ..
            } = calculate_page_table_index(9, 12, addr, 2, AddrWidth::Bits32);
            assert_eq!(
                l2_index, 0b111111111,
                "Should be 0b111111111 but is {l2_index:#b}",
            );
            let expected_bits: u64 = 0b111111111 << (9 + 12);
            assert_eq!(
                l2_bits, expected_bits,
                "Should be {l2_bits:#b} but is {expected_bits:#b}"
            );
        }

        {
            let PageTableLookupMetaInfo {
                index: l1_index,
                relevant_part_of_addr: l1_bits,
                ..
            } = calculate_page_table_index(9, 12, addr, 1, AddrWidth::Bits32);
            assert_eq!(
                l1_index, 0b010101010,
                "Should be 0b010101010 but is {l1_index:#b}",
            );
            let expected_bits: u64 = 0b010101010 << 12;
            assert_eq!(
                l1_bits, expected_bits,
                "Should be {l1_bits:#b} but is {expected_bits:#b}"
            );
        }
    }

    #[test]
    #[allow(clippy::identity_op)]
    fn test_calculate_page_table_index_64() {
        // a 64-bit address written so that it is separated by the corresponding
        // levels of page table on x86_64.
        #[allow(clippy::unusual_byte_groupings)]
        let addr = 0b000100000_000011111_111111111_010101010_001111000011;

        {
            let PageTableLookupMetaInfo {
                index: l4_index,
                relevant_part_of_addr: l4_bits,
                ..
            } = calculate_page_table_index(9, 12, addr, 4, AddrWidth::Bits64);
            assert_eq!(
                l4_index, 0b000100000,
                "Should be 0b000100000 but is {l4_index:#b}"
            );
            let expected_bits: u64 = 0b000100000 << (3 * 9 + 12);
            assert_eq!(
                l4_bits, expected_bits,
                "Should be {l4_bits:#b} but is {expected_bits:#b}"
            );
        }

        {
            let PageTableLookupMetaInfo {
                index: l3_index,
                relevant_part_of_addr: l3_bits,
                ..
            } = calculate_page_table_index(9, 12, addr, 3, AddrWidth::Bits64);
            assert_eq!(
                l3_index, 0b000011111,
                "Should be 0b000011111 but is {l3_index:#b}"
            );
            let expected_bits: u64 = 0b000011111 << (2 * 9 + 12);
            assert_eq!(
                l3_bits, expected_bits,
                "Should be {l3_bits:#b} but is {expected_bits:#b}"
            );
        }

        {
            let PageTableLookupMetaInfo {
                index: l2_index,
                relevant_part_of_addr: l2_bits,
                ..
            } = calculate_page_table_index(9, 12, addr, 2, AddrWidth::Bits64);
            assert_eq!(
                l2_index, 0b111111111,
                "Should be 0b111111111 but is {l2_index:#b}"
            );
            let expected_bits: u64 = 0b111111111 << (9 + 12);
            assert_eq!(
                l2_bits, expected_bits,
                "Should be {l2_bits:#b} but is {expected_bits:#b}"
            );
        }

        {
            let PageTableLookupMetaInfo {
                index: l1_index,
                relevant_part_of_addr: l1_bits,
                ..
            } = calculate_page_table_index(9, 12, addr, 1, AddrWidth::Bits64);
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
}
