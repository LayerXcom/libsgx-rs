#![cfg_attr(not(target_env = "sgx"), no_std)]

#[macro_use]
extern crate sgx_tstd as std;

#[cfg(debug_assertions)]
mod internal_tests {
    use sgx_tunittest::*;
    use crate::std::{panic::UnwindSafe, string::String, vec::Vec};

    pub unsafe fn internal_tests(ext_ptr: *const RawPointer) -> ResultStatus {
        let mut ctr = 0u64;
        let mut failures = Vec::new();
        rsgx_unit_test_start();

        // Add tests here
        // core_unitests(&mut ctr, &mut failures, app_msg_correctness, "app_msg_correctness");

        let result = failures.is_empty();
        rsgx_unit_test_end(ctr, failures);
        result.into()
    }

    fn core_unitests<F, R>(
        ncases: &mut u64,
        failurecases: &mut Vec<String>,
        f: F,
        name: &str
    )
    where
        F: FnOnce() -> R + UnwindSafe
    {
        *ncases = *ncases + 1;
        match std::panic::catch_unwind(|| { f(); }).is_ok()
        {
            true => {
                println!("{} {} ... {}!", "testing", name, "\x1B[1;32mok\x1B[0m");
            }
            false => {
                println!("{} {} ... {}!", "testing", name, "\x1B[1;31mfailed\x1B[0m");
                failurecases.push(String::from(name));
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ecall_run_tests(ext_ptr: *const RawPointer, result: *mut ResultStatus) {
    *result = ResultStatus::Ok;
    #[cfg(debug_assertions)]
    {
        let internal_tests_result = self::internal_tests::internal_tests(ext_ptr);
        *result = internal_tests_result;
    }
}
