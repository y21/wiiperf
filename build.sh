cargo b -p wiiperf --target powerpc-unknown-eabi.json -Zjson-target-spec -Zbuild-std=core,compiler_builtins -Zbuild-std-features=mem && \
    $DEVKITPRO/tools/bin/elf2dol target/powerpc-unknown-eabi/debug/wiiperf.elf wiiperf.dol
