use std::process::Command;
use clap::Parser;

#[derive(Debug, Parser)]
pub enum Options {
    BuildEbpf,
    Run {
        #[clap(trailing_var_arg = true, allow_hyphen_values = true)]
        run_args: Vec<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let opts = Options::parse();
    match opts {
        Options::BuildEbpf => build_ebpf()?,
        Options::Run { run_args } => {
            build_ebpf()?;
            let status = Command::new("cargo")
                .args(["run", "--package", "userspace", "--release", "--"])
                .args(run_args)
                .status()?;
            if !status.success() {
                anyhow::bail!("Failed to run userspace program");
            }
        }
    }
    Ok(())
}

fn build_ebpf() -> anyhow::Result<()> {
    println!("Building eBPF program...");
    let status = Command::new("cargo")
        .current_dir("bpf")
        .args([
            "+nightly",
            "build",
            "--target=bpfel-unknown-none",
            "-Z", "build-std=core",
            "--release",
        ])
        .status()?;
    
    if !status.success() {
        anyhow::bail!("Failed to build eBPF program");
    }
    Ok(())
}
