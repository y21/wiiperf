#![no_std]

use wiiperf::{exception, profiler};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wiiperf_install() {
    exception::install();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wiiperf_dump_profile() {
    profiler::dump_results();
}
