// bibiman - a TUI for managing BibLaTeX databases
// Copyright (C) 2024  lukeflo
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
/////

use sarge::prelude::*;
use std::env;
use std::path::PathBuf;

sarge! {
    // Name of the struct
    ArgumentsCLI,

    // Show help and exit.
    'h' help: bool,

    // Show version and exit.
    'v' version: bool,
}

// struct for CLIArgs
pub struct CLIArgs {
    pub helparg: bool,
    pub versionarg: bool,
    pub bibfilearg: Vec<PathBuf>,
}

impl Default for CLIArgs {
    fn default() -> Self {
        Self::new()
    }
}

impl CLIArgs {
    pub fn new() -> Self {
        let (cli_args, pos_args) = ArgumentsCLI::parse().expect("Could not parse CLI arguments");
        let bibfilearg = if pos_args.len() > 1 {
            parse_files(pos_args)
        } else {
            panic!("No bibfile provided")
        };
        Self {
            helparg: cli_args.help,
            versionarg: cli_args.version,
            bibfilearg,
        }
    }
}

pub fn parse_files(args: Vec<String>) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();
    for f in args {
        files.push(PathBuf::from(f))
    }
    files.remove(0);
    files
}

pub fn help_func() -> String {
    let help = format!(
        "\
{} {}

USAGE:
    bibiman [FLAGS] [file]

POSITIONAL ARGS:
    <file>    Path to .bib file

FLAGS:
    -h, --help      Show this help and exit
    -v, --version   Show the version and exit",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    );
    help
}

pub fn version_func() -> String {
    let version = format!(
        "\
{} {}
{}
{}

Target Triple: {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS"),
        env!("CARGO_PKG_LICENSE"),
        env!("TARGET")
    );
    version
}
