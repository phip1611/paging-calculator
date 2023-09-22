# Paging Address Calculator

`paging-calculator` is a CLI utility written in Rust that helps you find the indices that a
virtual address will have on different architectures or paging implementations.

It takes a (virtual) address in hexadecimal format and shows you which index will be used for what
page-table level. It can be installed with `$ cargo install paging-calculator`.

Valid inputs are:
- `$ paging-calculator 0x1337`
- `$ paging-calculator 0xdead_beef` (underscores are accepted)
- `$ paging-calculator 0xdead_beef x86 --pae` (underscores are accepted)
- `$ paging-calculator 0xdead_beef x86_64`

Type `$ paging-calculator help` to get a list of all supported options.

The following screenshot summarizes its functionality:

![Screenshot showing the usage of paging-calculator.](screenshot.png "Screenshot showing the usage of paging-calculator.")

# MSRV
The MSRV is `1.70.0` stable.

# Trivia
I worked on a project where I need to set up page-tables on my own. I had a few problems to find out
what I actually have to do and what indices are used at which level. With the help of this utility,
things are a little clearer to me now.
