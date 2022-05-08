use anyhow::Error;
use clap::Parser;

use application::Application;

use crate::args::Args;

mod application;
mod args;
mod package;
mod registry;
mod ui;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    let res = Application::new(args)?.run().await;

    if let Err(error) = res {
        eprintln!("{:?}", error)
    }

    Ok(())
}
