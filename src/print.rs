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

const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Whether ANSI escape sequences should be used or not.
pub static USE_ANSI: AtomicBool = AtomicBool::new(false);

use crate::addr_width::AddrWidth;
use crate::cli::{CliArgs, VirtualAddress};
use crate::page_table_index::PageTableLookupMetaInfo;
use crate::paging_info::PagingImplInfo;
use crate::print::ansi_styles::{paint_heading, paint_hint};
use std::sync::atomic::AtomicBool;

fn print_header(paging_info: &PagingImplInfo, v_addr: VirtualAddress) {
    print!(
        "{}",
        paint_heading(&format!(
            "Page Table Calculator (v{}): {}",
            CRATE_VERSION, paging_info.name
        ))
    );
    println!();
    println!("{}", paging_info.description);
    println!();
    if paging_info.addr_width == AddrWidth::Bits32 {
        println!(
            "address       : 0x{:x}  {info}",
            u64::from(v_addr) & 0xffffffff,
            info = paint_hint("(user input truncated to 32-bit)")
        );
        println!("address (bits): 0b{:032b}", u64::from(v_addr) & 0xffffffff);
    } else {
        println!("address       : {v_addr}");
        println!("address (bits): 0b{:064b}", u64::from(v_addr));
    }
}

/// Prints the information to the screen.
pub fn print(cli_input: &CliArgs) {
    let v_addr = cli_input.virtual_address;
    let paging_impl_info = PagingImplInfo::from_arch(cli_input.architecture);
    print_header(&paging_impl_info, v_addr);

    let page_table_lookup_info = paging_impl_info.calc_page_table_lookup_meta_info(v_addr);

    for info in page_table_lookup_info.iter().rev() {
        print!("level {} bits  : ", info.level);
        print_relevant_bits_highlighted(info, &paging_impl_info);
        println!();
    }

    for (is_first, info) in page_table_lookup_info
        .iter()
        .rev()
        .enumerate()
        .map(|(i, info)| (i == 0, info))
    {
        print!("level {} entry index : {:>6}", info.level, info.index);
        if is_first {
            print!("  {info}", info = paint_hint("(number of entry)"));
        }
        println!();

        print!(
            "level {} entry offset: 0x{:04x}",
            info.level,
            info.index * paging_impl_info.page_table_entry_size
        );
        if is_first {
            print!(
                "  {info}",
                info = paint_hint("(offset into the page table for that entry)")
            );
        }
        println!();
    }
}

// Prints the relevant bits used for the indexing and highlights them in red.
// Others are zeroed.
fn print_relevant_bits_highlighted(info: &PageTableLookupMetaInfo, paging_info: &PagingImplInfo) {
    let addr_width = u64::from(paging_info.addr_width);

    let zeroes_fill_right_count =
        paging_info.page_offset_bits + (info.level - 1) * paging_info.page_table_index_bits;

    let page_index_highlight_bits_count =
        if zeroes_fill_right_count + paging_info.page_table_index_bits > addr_width {
            addr_width - zeroes_fill_right_count
        } else {
            paging_info.page_table_index_bits
        };

    let zeroes_fill_left_count =
        addr_width - zeroes_fill_right_count - page_index_highlight_bits_count;

    print!(
        "0b{zeroes_left_fill}{highlighted_index}{zeroes_right_fill}",
        zeroes_left_fill = "0".repeat(zeroes_fill_left_count as usize),
        highlighted_index = ansi_styles::paint_highlight(&format!(
            "{index:0bits$b}",
            index = info.index,
            bits = page_index_highlight_bits_count as usize
        )),
        zeroes_right_fill = "0".repeat(zeroes_fill_right_count as usize)
    );
}

mod ansi_styles {
    use crate::print::USE_ANSI;
    use nu_ansi_term::{AnsiGenericString, Color, Style};
    use std::sync::atomic::Ordering;

    pub fn paint_highlight(str: &str) -> AnsiGenericString<'_, str> {
        if USE_ANSI.load(Ordering::SeqCst) {
            Style::new().fg(Color::Red).bold().paint(str)
        } else {
            Style::new().paint(str)
        }
    }

    pub fn paint_heading(str: &str) -> AnsiGenericString<'_, str> {
        if USE_ANSI.load(Ordering::SeqCst) {
            Style::new().bold().paint(str)
        } else {
            Style::new().paint(str)
        }
    }

    pub fn paint_hint(str: &str) -> AnsiGenericString<'_, str> {
        if USE_ANSI.load(Ordering::SeqCst) {
            Style::new().fg(Color::LightGray).paint(str)
        } else {
            Style::new().paint(str)
        }
    }
}
