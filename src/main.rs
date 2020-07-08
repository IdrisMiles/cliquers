use cliquers;
use std::error::Error;
use std::fs;
use std::io;
use std::path;
use structopt::StructOpt;
use walkdir::WalkDir;

// List files grouping filesequences together.
#[derive(StructOpt)]
struct Cli {
    /// Print files not in a collection
    #[structopt(short = "s", long = "show-remainder")]
    show_remainder: bool,

    /// Recurse down subdirectories
    #[structopt(short = "r", long = "recurse")]
    recurse: bool,

    /// Optional format of filesequences, default format: "{head}{padding}{tail} [{ranges}]"
    #[structopt(short = "f", long = "format")]
    format: Option<String>,

    /// Optional custom pattern for grouping collections of files, default pattern: "(?P<index>(?P<padding>0*)\d+)"
    #[structopt(short = "p", long = "patterns")]
    patterns: Option<Vec<String>>,

    /// The path to list files and filesequences under
    #[structopt(parse(from_os_str))]
    paths: Vec<path::PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::from_args();

    match args.recurse {
        true => {
            for path in args.paths {
                for dir in WalkDir::new(path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter_map(|e| match e.file_type().is_dir() {
                        true => Some(e),
                        false => None,
                    })
                {
                    let entries = fs::read_dir(dir.path())?
                        .map(|res| res.map(|e| e.path().to_str().unwrap().to_string()))
                        .collect::<Result<Vec<_>, io::Error>>()?;

                    let (collections, remainders) =
                        cliquers::assemble(&entries, args.patterns.to_owned());
                    for c in collections.iter() {
                        println!("{}", c.format(args.format.to_owned()));
                    }
                    if args.show_remainder {
                        for r in remainders.iter() {
                            println!("{}", r);
                        }
                    }
                }
            }
        }
        false => {
            for path in args.paths {
                let entries = fs::read_dir(path)?
                    .map(|res| res.map(|e| e.path().to_str().unwrap().to_string()))
                    .collect::<Result<Vec<_>, io::Error>>()?;

                let (collections, remainders) =
                    cliquers::assemble(&entries, args.patterns.to_owned());
                for c in collections.iter() {
                    println!("{}", c.format(args.format.to_owned()));
                }
                if args.show_remainder {
                    for r in remainders.iter() {
                        println!("{}", r);
                    }
                }
            }
        }
    }

    Ok(())
}
