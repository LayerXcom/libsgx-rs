#![no_std]

#[macro_use]
extern crate sgx_tstd as std;

mod client;
mod quote;

pub use crate::client::{RAService, AttestationReport, ReportSig};
