#![cfg(target_os = "linux")]
#![doc = include_str!("../README.md")]
#![doc(html_root_url = "https://docs.rs/smartcols-sys/0.1.3")]
#![allow(non_camel_case_types)]

#[cfg(test)]
mod tests;

use std::os::raw::c_uint;

pub const SCOLS_DEBUG_HELP: c_uint = 1 << 0;
pub const SCOLS_DEBUG_INIT: c_uint = 1 << 1;
pub const SCOLS_DEBUG_CELL: c_uint = 1 << 2;
pub const SCOLS_DEBUG_LINE: c_uint = 1 << 3;
pub const SCOLS_DEBUG_TAB: c_uint = 1 << 4;
pub const SCOLS_DEBUG_COL: c_uint = 1 << 5;
pub const SCOLS_DEBUG_BUFF: c_uint = 1 << 6;
pub const SCOLS_DEBUG_GROUP: c_uint = 1 << 7;
pub const SCOLS_DEBUG_FLTR: c_uint = 1 << 8;
pub const SCOLS_DEBUG_FPARAM: c_uint = 1 << 9;
pub const SCOLS_DEBUG_ALL: c_uint = 0xFFFF;

include!(concat!(env!("OUT_DIR"), "/smartcols-sys.rs"));
