#![no_std]

#[macro_use]
extern crate sgx_tstd as std;

mod report;
mod client;

pub use crate::client::{RAService, Report, ReportSig};
