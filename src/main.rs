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

    /// compiler --release
    #[clap(long, short)]
    release: bool,

    /// compiler --bin
    #[clap(long, short)]
    bin: String,

    /// compiler --target, if defined use cross compiler(disable RUSTFLAGS)
    #[clap(long, short)]
    target: Option<String>,
}

fn main() {
    let opts = Opts::parse();

    // choose target compiler
    let compiler = if opts.target.is_some() {
        "cross"
    } else {
        "cargo"
    };

    let mut cargo_build = Command::new(compiler);
    cargo_build.arg("build");
    cargo_build.arg("--workspace");

    // bin
    cargo_build.arg("--bin");
    cargo_build.arg(&opts.bin);

    // release
    let mode = if opts.release {
        cargo_build.arg("--release");
        "release"
    } else {
        "debug"
    };

    let target_path = if let Some(ref target) = opts.target {
        cargo_build.arg("--target");
        cargo_build.arg(target);
        format!("target/{}", target)
    } else {
        "target".to_string()
    };

    // Read the RUSTFLAGS environment variable
    if opts.target.is_none() {
        let rustflags = ::std::env::var_os("RUSTFLAGS")
            .unwrap_or_default()
            .into_string()
            .expect("RUSTFLAGS are not valid UTF-8");
        cargo_build.env("RUSTFLAGS", rustflags);
    }

    println!("[-] running: {:?}", cargo_build);
    let output = cargo_build.output().unwrap();
    println!("{:?}", output);

    let binpath = format!("./{}/{}/{}", target_path, mode, opts.bin);

    println!("{binpath}");
    let mut rz = RzPipe::spawn(binpath, None).unwrap();

    let _ = rz.cmd("aa").unwrap();
    let output = rz.cmd(&format!("pdg @ $(afl~{}[0])", opts.s)).unwrap();
    println!("{}", output);

    rz.close();
}
