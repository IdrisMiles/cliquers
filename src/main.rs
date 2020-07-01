use cliquers;
use std::error::Error;
use std::fs;
use std::io;
use std::path;
use structopt::StructOpt;

// List files grouping filesequences together.
#[derive(StructOpt)]
struct Cli {
    /// Optional format of filesequences, default format: "{head}{padding}{tail} [{ranges}]"
    #[structopt(short = "f", long = "format")]
    format: Option<String>,

    /// Optional custom pattern for grouping collections of files, default pattern: "(?P<index>(?P<padding>0*)\d+)"
    #[structopt(short = "p", long = "patterns")]
    patterns: Option<Vec<String>>,

    /// The path to list files and filesequences under
    #[structopt(parse(from_os_str))]
    path: path::PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::from_args();

    let entries = fs::read_dir(args.path)?
        .map(|res| res.map(|e| e.path().to_str().unwrap().to_string()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    let (collections, remainders) = cliquers::assemble(&entries, args.patterns);

    for c in collections.iter() {
        println!("{}", c.format(args.format.to_owned()));
    }
    for r in remainders.iter() {
        println!("{}", r);
    }

    Ok(())
}
