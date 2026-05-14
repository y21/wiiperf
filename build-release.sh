cargo b -r -p wiiperf-example --target "powerpc-unknown-eabi.json"  -Zjson-target-spec && \
    $DEVKITPRO/tools/bin/elf2dol target/powerpc-unknown-eabi/release/wiiperf-example.elf wiiperf.dol
