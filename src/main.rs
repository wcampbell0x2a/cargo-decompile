use std::{process::Command, str::FromStr};

use clap::Parser;
use r2pipe::R2Pipe;
use rzpipe::RzPipe;
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

#[derive(Debug, Clone, Parser)]
enum Tool {
    Rizin,
    Radare2,
}

impl FromStr for Tool {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rizin" => Ok(Self::Rizin),
            "radare2" => Ok(Self::Radare2),
            _ => Err("unknown tool".to_string()),
        }
    }
}

#[derive(Debug, Clone, Parser)]
struct Opts {
    /// function symbol used in rizin for ghidra decompile
    #[clap(long, short)]
    s: String,

    /// compiler --release
    #[clap(long)]
    release: bool,

    /// compiler --bin
    #[clap(long)]
    bin: String,

    /// compiler --target, if defined use cross compiler(disable RUSTFLAGS)
    #[clap(long)]
    target: Option<String>,

    /// rizin or radare2
    #[clap(long)]
    tool: Tool,
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

    let cmd = format!("pdg @ $(afl~{}[0])", opts.s);

    let mut output = None;
    match opts.tool {
        Tool::Rizin => {
            let mut rz = RzPipe::spawn(binpath, None).unwrap();

            let _ = rz.cmd("aa").unwrap();
            output = Some(rz.cmd(&cmd).unwrap());

            rz.close();
        }
        Tool::Radare2 => {
            let mut r2 = R2Pipe::spawn(binpath, None).unwrap();

            let _ = r2.cmd("aa").unwrap();
            output = Some(r2.cmd(&cmd).unwrap());

            r2.close();
        }
    }
    println!("{:?}", output);
    if let Some(output) = output {
        // Load these once at the start of your program
        let ps = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        let syntax = ps.find_syntax_by_extension("c").unwrap();
        let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
        for line in LinesWithEndings::from(&output) { // LinesWithEndings enables use of newlines mode
            let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
            let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
            print!("{}", escaped);
        }
    }

}
