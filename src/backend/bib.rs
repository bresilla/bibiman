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

use biblatex::Bibliography;
use std::{fs, path::PathBuf};

use super::cliargs::PosArgs;

// Set necessary fields
// TODO: can surely be made more efficient/simpler
pub struct BibiMain {
    pub bibfile: PathBuf,           // path to bibfile
    pub bibfilestring: String,      // content of bibfile as string
    pub bibliography: Bibliography, // parsed bibliography
    pub citekeys: Vec<String>,      // list of all citekeys
}

impl BibiMain {
    pub fn new() -> Self {
        // TODO: Needs check for config file path as soon as config file is impl
        let bibfile = PosArgs::parse_pos_args().bibfilearg;
        let bibfilestring = fs::read_to_string(&bibfile).unwrap();
        let bibliography = biblatex::Bibliography::parse(&bibfilestring).unwrap();
        let citekeys = Self::get_citekeys(&bibliography);
        Self {
            bibfile,
            bibfilestring,
            bibliography,
            citekeys,
        }
    }

    // get list of citekeys from the given bibfile
    // this list is the base for further operations on the bibentries
    // since it is the entry point of the biblatex crate.
    pub fn get_citekeys(bibstring: &Bibliography) -> Vec<String> {
        let mut citekeys: Vec<String> =
            bibstring.iter().map(|entry| entry.to_owned().key).collect();
        citekeys.sort_by_key(|name| name.to_lowercase());
        citekeys
    }
}

pub struct BibiData {
    pub citekey: Vec<String>,
}
