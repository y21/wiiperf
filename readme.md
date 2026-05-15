# wiiperf

A CPU sampling profiler for the Wii.

> ⚠️ Stability status: "works on my dolphin" (works but expect rough edges)

### Building
This project requires a pinned nightly Rust toolchain (see rust-toolchain.toml) as well as devkitpro build tools (powerpc-eabi-gcc, elf2dol). 

There are two targets you can build: 

- `cargo xtask build example --release`: builds an example DOL that you can run in Dolphin. The DOL will be at the workspace root. `--release` is optional, will be faster but omitting it can be easier to debug.
You can play around with the main function and add some code; it will continuously print results as it collects samples. 

- `cargo xtask build lib --release`: creates a self-contained `.a` archive containing the library in `target/powerpc-unknown-eabi/{release,debug}/libwiiperf.a`. This can be linked together with any other project and is the main way to use this. 

### Usage
After building the library as shown in the previous section, you can add `-Lpath-to-archive -lwiiperfc` to your build command to link it in and start by calling `wiiperf_install();`
See wiiperfc/capi.h for the full API and docs. 

### How it works
wiiperf uses the decrementer cpu register to trigger frequent interrupts. It registers a decrementer exception handler to collects SRR0 values (addresses at which the interrupt occurs), and sorts if by frequency.

### Limitations
- this register is often also used for timers (e.g. if you're using libogc, it registers its own decrementer handler which creates a conflict). wiiperf tries to "proxy" any existing handlers and it should still work, but the code you're integrating wiiperf into might adjust the decrementer in ways that can make sampling inaccurate.
- the `.a` archive currently contains various symbols that libgcc normally ships which can result in your code calling not into libgcc's routines but this library's. Most of these shouldn't make a difference, but e.g. `__divdi3` traps when previously it might not.
- if threads are involved, interpreting it may be harder or potentially misleading.
