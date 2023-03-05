/*
MIT License

Copyright (c) 2023 Philipp Schuster

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
//! Module for specific paging implementations.

use crate::addr_width::AddrWidth;
use crate::cli::{Architecture, VirtualAddress};
use crate::page_table_index::{calculate_page_table_index, PageTableLookupMetaInfo};

#[derive(Debug)]
pub struct PagingImplInfo {
    /// Short name of the paging implementation.
    pub name: &'static str,
    /// Descriptive text of the paging implementation.
    pub description: &'static str,
    /// Address width.
    pub addr_width: AddrWidth,
    /// Number of bits used to index into the page. 2 to the power of this value
    /// equals the page size.
    pub page_offset_bits: u64,
    /// Number of bits used to index into a page table. 2 to the power of this
    /// value equals the number of entries per page table. This implementation
    /// relies on the fact that the amount of bits indexing a page-table do not
    /// dynamically vary in the middle of the address, which is not done by any
    /// paging implementation luckily.
    pub page_table_index_bits: u64,
    /// Size of a page table entry in bytes.
    pub page_table_entry_size: u64,
    /// Number of page-table levels.
    pub levels: u64,
}

impl PagingImplInfo {
    /// Const constructor for [`PagingImplInfo`] from [`Architecture`]. Returns one
    /// of the constants of the [`impls`] module.
    pub const fn from_arch(arch: Architecture) -> Self {
        match arch {
            Architecture::X86 { pae: false, .. } => impls::X86,
            Architecture::X86 { pae: true, .. } => impls::X86_PAE,
            Architecture::X86_64 {
                five_level: false, ..
            } => impls::X86_64,
            Architecture::X86_64 {
                five_level: true, ..
            } => impls::X86_64_5LEVEL,
        }
    }

    /// Calculates the [`PageTableLookupMetaInfo`] for all levels for a virtual
    /// address and the given paging [`PagingImplInfo`]. The amount of results
    /// corresponds to the amount of page-table levels. The first element
    /// corresponds to level 1 and the last element to level n.
    pub fn calc_page_table_lookup_meta_info(
        &self,
        v_addr: VirtualAddress,
    ) -> Vec<PageTableLookupMetaInfo> {
        let mut level = 0;
        let mut level_info_vec = vec![];
        while level < self.levels {
            level += 1;
            let info = calculate_page_table_index(
                self.page_table_index_bits,
                self.page_offset_bits,
                v_addr,
                level,
                self.addr_width,
            );
            level_info_vec.push(info);
        }
        level_info_vec
    }
}

pub mod impls {
    use super::*;
    use std::mem::size_of;

    pub const X86: PagingImplInfo = PagingImplInfo {
        name: "x86 32-bit paging",
        levels: 2,
        description: "x86 paging uses a 2-level page table. The page is indexed by 12 bits,\n\
            which results in a page-size of 4096 bytes. Each page table is indexed by 10\n\
            bits and has 2^10 == 1024 entries. Each page-table entry is 32-bit in size.\n\
            Hence, a page table occupies the size of a page. Huge pages have a size of\n\
            2^22 == 4 MiB.",
        addr_width: AddrWidth::Bits32,
        page_offset_bits: 12,
        page_table_index_bits: 10,
        page_table_entry_size: size_of::<u32>() as u64,
    };

    pub const X86_PAE: PagingImplInfo = PagingImplInfo {
        name: "x86 32-bit paging with PAE",
        levels: 3,
        description:
            "x86 with the Physical Address Extension (PAE) paging uses a 3-level page table,\n\
            that enables to access more than 32-bit of physical address space. The page\n\
            is indexed by 12 bits, which results in a page-size of 4096 bytes. Tables\n\
            at level 1 and 2 are indexed by 9 bits and have 2^9 == 512 entries. The third-\n\
            level page table is indexed by 2 bits and has 2^2 == 4 entries. Each page-table\n\
            entry is 64-bit in size. Hence, a page table at levels 1 and 2 occupies the size\n\
            of a page whereas the level 3 page table occupies 32 byte. Huge pages have a size\n\
            of 2^21 == 2 MiB and are only valid on level 2.",
        addr_width: AddrWidth::Bits32,
        page_offset_bits: 12,
        page_table_index_bits: 9,
        page_table_entry_size: size_of::<u64>() as u64,
    };

    pub const X86_64: PagingImplInfo = PagingImplInfo {
        name: "x86_64 paging",
        levels: 4,
        description: "x86_64 paging uses a 4-level page table. The page is indexed by 12 bits,\n\
            which results in a page-size of 4096 bytes. Each page table is indexed by 9\n\
            bits and has 2^9 == 512 entries. Each page-table entry is 64-bit in size. Hence,\n\
            a page table occupies the size of a page. Huge pages have a size of\n\
            2^21 == 2 MiB or 2^30 == 1 GiB. Huge pages are only valid on levels 2 or 3.",
        addr_width: AddrWidth::Bits64,
        page_offset_bits: 12,
        page_table_index_bits: 9,
        page_table_entry_size: size_of::<u64>() as u64,
    };

    pub const X86_64_5LEVEL: PagingImplInfo = PagingImplInfo {
        name: "x86_64 paging (5-level)",
        levels: 5,
        description: "x86_64 paging optionally uses a 5-level page table. The page is indexed\n\
            by 12 bits, which results in a page-size of 4096 bytes. Each page table is\n\
            indexed by 9 bits and has 2^9 == 512 entries. Each page-table entry is 64-bit in\n\
            size. Hence, a page table occupies the size of a page. Huge pages have a size of\n\
            2^21 == 2 MiB or 2^30 == 1 GiB. Huge pages are only valid on levels 2 or 3.",
        addr_width: AddrWidth::Bits64,
        page_offset_bits: 12,
        page_table_index_bits: 9,
        page_table_entry_size: size_of::<u64>() as u64,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_page_table_lookup_meta_info_x86() {
        // a 32-bit address written so that it is seperated by the corresponding
        // levels of page table on x86.
        #[allow(clippy::unusual_byte_groupings)]
        let addr = 0b1111111111_1010101010_001111000011.into();

        let vec = impls::X86.calc_page_table_lookup_meta_info(addr);
        assert_eq!(vec[0].index, 0b1010101010);
        assert_eq!(vec[1].index, 0b1111111111);
        assert_eq!(vec.len(), 2);
    }

    #[test]
    fn test_calc_page_table_lookup_meta_info_x86_pae() {
        // a 32-bit address written so that it is seperated by the corresponding
        // levels of page table on x86 with PAE.
        #[allow(clippy::unusual_byte_groupings)]
        let addr = 0b10_111111111_010101010_001111000011.into();

        let vec = impls::X86_PAE.calc_page_table_lookup_meta_info(addr);
        assert_eq!(vec[0].index, 0b010101010);
        assert_eq!(vec[1].index, 0b111111111);
        assert_eq!(vec[2].index, 0b10);
        assert_eq!(vec.len(), 3);
    }

    #[test]
    fn test_calc_page_table_lookup_meta_info_x86_64() {
        // a 64-bit address written so that it is separated by the corresponding
        // levels of page table on x86_64.
        #[allow(clippy::unusual_byte_groupings)]
        let addr = 0b000100000_000011111_111111111_010101010_001111000011.into();

        let vec = impls::X86_64.calc_page_table_lookup_meta_info(addr);
        assert_eq!(vec[0].index, 0b010101010);
        assert_eq!(vec[1].index, 0b111111111);
        assert_eq!(vec[2].index, 0b000011111);
        assert_eq!(vec[3].index, 0b000100000);
        assert_eq!(vec.len(), 4);
    }

    #[test]
    fn test_calc_page_table_lookup_meta_info_x86_64_4level() {
        // a 64-bit address written so that it is separated by the corresponding
        // levels of page table on x86_64.
        #[allow(clippy::unusual_byte_groupings)]
        let addr = 0b011101110_000100000_000011111_111111111_010101010_001111000011.into();

        let vec = impls::X86_64_5LEVEL.calc_page_table_lookup_meta_info(addr);
        assert_eq!(vec[0].index, 0b010101010);
        assert_eq!(vec[1].index, 0b111111111);
        assert_eq!(vec[2].index, 0b000011111);
        assert_eq!(vec[3].index, 0b000100000);
        assert_eq!(vec[4].index, 0b011101110);
        assert_eq!(vec.len(), 5);
    }
}
