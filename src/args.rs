use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    /// Path to a directory that contains a package.json file
    #[clap(parse(from_os_str))]
    pub directory: Option<PathBuf>,
}
