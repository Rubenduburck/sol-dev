extern crate ansi_term;
extern crate rayon;
use self::ansi_term::Colour::{Cyan, Green, Red};
pub use self::error::Error;
use self::rayon::iter::{IntoParallelIterator as _, ParallelIterator as _};

use clap::Parser;

mod consumption;
pub mod error;
mod function;
mod invoke;
mod log;

#[derive(Parser)]
pub struct Cli {
    #[clap(subcommand)]
    cmd: Command,
}

impl Cli {
    pub fn run(&self) -> Result<(), Error> {
        self.cmd.run()
    }
}

#[derive(clap::Parser)]
pub struct Args {
    #[clap(short, long)]
    pub output: Option<String>,

    #[clap(short, long, default_value = "_parsed")]
    pub postfix: String,

    pub path: String,
}

#[derive(clap::Parser)]
pub enum Command {
    File(Args),
    Dir(Args),
}

impl Command {
    fn parse_and_write(&self, infile: &str) -> Result<(), Error> {
        let outfile = self.outfile(infile)?;
        std::fs::write(
            &outfile,
            serde_json::to_string_pretty(&log::Log::from_slice(
                &serde_json::from_str::<Vec<String>>(&std::fs::read_to_string(infile)?)?
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>(),
            ))?,
        )?;
        println!("{} {}", Green.paint("Wrote"), Cyan.paint(outfile));
        Ok(())
    }

    fn files(&self) -> Result<Vec<String>, Error> {
        match self {
            Command::File(args) => Ok(vec![args.path.clone()]),
            Command::Dir(args) => Ok(std::fs::read_dir(args.path.clone())?
                .map(|p| p.unwrap().path().to_str().unwrap().to_string())
                .filter(|p| p.ends_with(".json") && !p.ends_with("_parsed.json"))
                .collect::<Vec<_>>()),
        }
    }

    fn outfile(&self, infile: &str) -> Result<String, Error> {
        Ok(match self {
            Command::File(args) => args
                .output
                .clone()
                .unwrap_or_else(|| format!("{}{}.json", infile.replace(".json", ""), args.postfix)),
            Command::Dir(args) => {
                if let Some(output) = &args.output {
                    let input_filename = std::path::Path::new(&infile)
                        .file_name()
                        .ok_or_else(|| {
                            std::io::Error::new(std::io::ErrorKind::Other, "No filename")
                        })?
                        .to_str()
                        .ok_or_else(|| {
                            std::io::Error::new(std::io::ErrorKind::Other, "Invalid filename")
                        })?;
                    format!("{}/{}", output, input_filename)
                } else {
                    format!("{}{}.json", infile.replace(".json", ""), args.postfix)
                }
            }
        })
    }

    pub fn run(&self) -> Result<(), Error> {
        self.files()?.into_par_iter().for_each(|filename| {
            if let Err(e) = self.parse_and_write(&filename) {
                println!("{} {}: {}", Red.paint("Error"), filename, e);
            }
        });
        Ok(())
    }
}
