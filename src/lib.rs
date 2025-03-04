#![cfg(target_os = "linux")]
#![doc = include_str!("../README.md")]
#![doc(html_root_url = "https://docs.rs/smartcols-sys/0.1.1")]
#![allow(non_camel_case_types)]

#[cfg(test)]
mod tests;

include!(concat!(env!("OUT_DIR"), "/smartcols-sys.rs"));
