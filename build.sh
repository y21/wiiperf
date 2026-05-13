cargo b -p wiiperf-example -Zjson-target-spec && \
    $DEVKITPRO/tools/bin/elf2dol target/powerpc-unknown-eabi/debug/wiiperf.elf wiiperf.dol
