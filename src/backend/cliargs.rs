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

use core::panic;
use std::path::{Path, PathBuf};

use sarge::prelude::*;

sarge! {
    // Name of the struct
    ArgumentsCLI,

    // Show help and exit.
    'h' help: bool,

    // Show version and exit. TODO: Write version...
    'v' version: bool,

    // Option for file: -b - short option; --bibfile - long option
    // #ok makes it optional
    #ok 'b' bibfile: String,
}

// struct for CLIArgs
pub struct CLIArgs {
    pub helparg: bool,
    pub versionarg: bool,
}

impl CLIArgs {
    pub fn parse_cli_args() -> Self {
        let (cli_args, _) = ArgumentsCLI::parse().expect("Could not parse CLI arguments");
        Self {
            helparg: cli_args.help,
            versionarg: cli_args.version,
        }
    }
}

// Struct for positional arguments
// TODO: Can surely be improved!!
pub struct PosArgs {
    pub bibfilearg: PathBuf,
}

impl PosArgs {
    pub fn parse_pos_args() -> Self {
        let (_, pos_args) = ArgumentsCLI::parse().expect("Could not parse positional arguments");
        Self {
            bibfilearg: if pos_args.len() > 1 {
                PathBuf::from(&pos_args[1])
                // pos_args[1].to_string()
            } else {
                panic!("No path to bibfile provided as argument")
            }, // bibfilearg: pos_args[1].to_string(),
        }
    }
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
{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS"),
        env!("CARGO_PKG_LICENSE")
    );
    version
}
