#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Installs the exception handler and starts collecting profiles.
 */
void wiiperf_install(void);

/**
 * Manually dumps the current profiler results via OSReport().
 */
void wiiperf_dump_profile(void);
