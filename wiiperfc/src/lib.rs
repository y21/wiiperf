#![no_std]

use wiiperf::{exception, profiler};

/// Installs the exception handler and starts collecting profiles.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wiiperf_install() {
    exception::install();
}

/// Manually dumps the current profiler results via OSReport().
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wiiperf_dump_profile() {
    profiler::dump_results();
}
