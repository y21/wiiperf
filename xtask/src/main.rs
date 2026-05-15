use std::process::Command;

use anyhow::{Context, bail};
use pico_args::Arguments;

fn main() -> anyhow::Result<()> {
    let mut args = Arguments::from_env();

    match &*args
        .free_from_str::<String>()
        .context("failed to parse command")?
    {
        "build" => {
            let release = args.contains("--release");

            match &*args
                .free_from_str::<String>()
                .context("failed to parse target")?
            {
                "lib" => {
                    build_binary(release, "wiiperfc").context("failed to build lib target")?;
                }
                "example" => {
                    build_target(release, "wiiperf-example")
                        .context("failed to build example target")?;
                }
                target => bail!("unknown build target: {target}"),
            }
        }
        "gen-header" => gen_header().context("failed to generate headers")?,
        cmd => bail!("unknown command: {cmd}"),
    }

    Ok(())
}

fn gen_header() -> anyhow::Result<()> {
    Command::new("cbindgen")
        .args([
            "--crate",
            "wiiperfc",
            "--lang",
            "c",
            "--output",
            "wiiperfc/capi.h"
        ])
        .status()
        .context("failed to spawn cbindgen")?;
    
    Ok(())
}

fn build_target(release: bool, package: &str) -> anyhow::Result<()> {
    build_binary(release, package)?;
    build_dol(release, package)?;

    Ok(())
}

fn build_binary(release: bool, package: &str) -> anyhow::Result<()> {
    let mut cargo = Command::new("cargo");
    cargo.args([
        "build",
        "-p",
        package,
        "--all-features",
        "--target",
        "powerpc-unknown-eabi.json",
        "-Zjson-target-spec",
        "-Zbuild-std=core,compiler_builtins",
        "-Zbuild-std-features=mem,optimize_for_size",
    ]);
    if release {
        cargo.arg("--release");
    }
    cargo.status().context("failed to build elf")?;

    Ok(())
}

fn build_dol(release: bool, package: &str) -> anyhow::Result<()> {
    Command::new("elf2dol")
        .args([
            &format!(
                "target/powerpc-unknown-eabi/{}/{}.elf",
                if release { "release" } else { "debug" },
                package
            ),
            &format!("{}.dol", package),
        ])
        .status()
        .context("failed to convert elf to dol")?;
    Ok(())
}
