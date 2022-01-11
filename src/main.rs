use clap::Parser;
use rzpipe::RzPipe;
use std::process::Command;

// TODO: support --target

#[derive(Debug, Clone, Parser)]
struct Opts {
    /// since this is a clap sub-command, this is always rz-ghidra
    #[clap(hide(true))]
    _name: String,

    /// function symbol used in rizin for ghidra decompile
    #[clap(long, short)]
    s: String,

    /// cargo --release
    #[clap(long, short)]
    release: bool,

    /// cargo --bin
    #[clap(long, short)]
    bin: String,
}

fn main() {
    let opts = Opts::parse();

    let mut cargo_build = Command::new("cargo");
    cargo_build.arg("build");
    cargo_build.arg("--bin");
    cargo_build.arg(&opts.bin);
    let mode = if opts.release {
        cargo_build.arg("--release");
        "release"
    } else {
        "debug"
    };

    // Read the RUSTFLAGS environment variable
    let rustflags = ::std::env::var_os("RUSTFLAGS")
        .unwrap_or_default()
        .into_string()
        .expect("RUSTFLAGS are not valid UTF-8");
    cargo_build.env("RUSTFLAGS", rustflags);

    let _ = cargo_build.output().unwrap();

    let binpath = format!("./target/{}/{}", mode, opts.bin);

    let mut rz = RzPipe::spawn(binpath, None).unwrap();

    let _ = rz.cmd("aa").unwrap();
    let output = rz.cmd(&format!("pdg @ $(is~{}[1])", opts.s)).unwrap();
    println!("{}", output);

    rz.close();
}
