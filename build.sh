cargo b -p wiiperf-example --all-features --target "powerpc-unknown-eabi.json" -Zjson-target-spec && \
    $DEVKITPRO/tools/bin/elf2dol target/powerpc-unknown-eabi/debug/wiiperf-example.elf wiiperf.dol
