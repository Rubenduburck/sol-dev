extern crate clap;
extern crate tracing_subscriber;
mod parser;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Parser error: {0}")]
    Parse(#[from] parser::Error),
}

use clap::Parser;

pub fn init_env_logger() {
    use tracing_subscriber::fmt::format::FmtSpan;
    use tracing_subscriber::prelude::*;

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_span_events(FmtSpan::CLOSE)
        .pretty();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(fmt_layer)
        .init();
}

#[derive(clap::Parser)]
pub enum Command {
    Parse(parser::Cli),
}

impl Command {
    pub fn run(&self) -> Result<(), Error> {
        match &self {
            Command::Parse(cmd) => cmd.run()?,
        }
        Ok(())
    }
}

#[derive(clap::Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub cmd: Command,
}

impl Cli {
    pub fn run(&self) -> Result<(), Error> {
        match &self.cmd {
            Command::Parse(cmd) => Ok(cmd.run()?),
        }
    }
}

fn main() {
    init_env_logger();
    if let Err(e) = Cli::parse().cmd.run() {
        tracing::error!("Error: {}", e);
    }
}
