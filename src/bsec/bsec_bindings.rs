//! Rust bindings for the BSEC library.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(missing_docs)]
#![allow(clippy::unreadable_literal)]

include!(concat!(env!("OUT_DIR"), "/bsec_bindings.rs"));
