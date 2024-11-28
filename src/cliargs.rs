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

use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use lexopt::prelude::*;
use std::env;
use std::path::PathBuf;
use walkdir::WalkDir;

// struct for CLIArgs
#[derive(Debug, Default, Clone)]
pub struct CLIArgs {
    pub helparg: bool,
    pub versionarg: bool,
    pub pos_args: Vec<PathBuf>,
    pub files: Vec<PathBuf>,
}

impl CLIArgs {
    pub fn parse_args() -> Result<CLIArgs, lexopt::Error> {
        let mut args = CLIArgs::default();
        let mut parser = lexopt::Parser::from_env();

        while let Some(arg) = parser.next()? {
            match arg {
                Short('h') | Long("help") => args.helparg = true,
                Short('v') | Long("version") => args.versionarg = true,
                // Value(pos_arg) => parse_files(&mut args, pos_arg),
                Value(pos_arg) => args.pos_args.push(pos_arg.into()),
                _ => return Err(arg.unexpected()),
            }
        }

        args.files = parse_files(args.pos_args.clone());

        Ok(args)
    }
}

/// This function maps a vector containing paths to another vector containing paths.
/// But it will walk all entries of the first vec which are directories
/// and put only valid file paths with `.bib` ending to the resulting vec.
fn parse_files(args: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();
    // If pos arg is file, just push it to path vec
    for i in args {
        if i.is_file() {
            files.push(i);
        // If pos arg is dir, walk dir and collect bibfiles
        } else if i.is_dir() {
            for file in WalkDir::new(i) {
                let f = file.unwrap().into_path();
                if f.is_file()
                    && f.extension().is_some()
                    && f.extension().unwrap_or_default() == "bib"
                {
                    files.push(f)
                }
            }
        } else {
            println!(
                "{}\n{}",
                "The positional argument is neither a valid file, nor a directory:"
                    .red()
                    .bold(),
                i.as_os_str().to_string_lossy().bright_red().italic()
            );
            println!();
            println!("{}", help_func());
            std::process::exit(1)
        }
    }
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
