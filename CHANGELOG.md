# Changelog for Paging Address Calculator / `paging-calculator`

## v0.4.0
- The binary is now much smaller (637 KiB on Linux instead of 883 KiB)

## v0.3.0 (2023-09-22)
- **BREAKING** The MSRV is `1.70.0` stable.
- updated dependencies

## v0.2.0 (2023-03-05)
- more functionality, such as `$ paging-calculator 0xdeadbeef x86 --pae` to get
  page table indices for x86 with the physical address extension (PAE). Type
  `$ paging-calculator help` for more instructions.
- MSRV is `1.64.0`

## v0.1.2 (2022-12-01)
- minor internal fixes

## v0.1.1 (2022-12-01)
- output both, entry index and entry offset
- print 32-bit addresses only with 32-bits and not 64-bits

## v0.1.0 (2022-11-30)
- initial release
