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

use super::cliargs::PosArgs;
use std::{fs, path::PathBuf};

use biblatex::PermissiveType;
use biblatex::{self, Bibliography};
use biblatex::{ChunksExt, Type};

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
        // let bibentries = BibiDataSets::from_iter(citekeys.clone());
        Self {
            bibfile,
            bibfilestring,
            bibliography,
            citekeys,
            // bibentries,
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

#[derive(Debug)]
pub struct BibiData {
    pub entry_list: BibiDataSets,
}

impl BibiData {
    pub fn new() -> Self {
        let citekeys = BibiMain::new().citekeys;
        Self {
            entry_list: BibiDataSets::from_iter(citekeys),
        }
    }
}

// Parent struct which keeps the Vector of all bibentries
// Necessary for implementing FromIterator
#[derive(Debug)]
pub struct BibiDataSets {
    pub bibentries: Vec<BibiEntry>,
}

impl FromIterator<String> for BibiDataSets {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        let bibentries = iter
            .into_iter()
            .map(|citekey| BibiEntry::new(&citekey))
            .collect();
        Self { bibentries }
    }
}

// Struct which has to be created for every entry of bibdatabase
#[derive(Debug)]
pub struct BibiEntry {
    pub authors: String,
    pub title: String,
    pub year: String,
    pub pubtype: String,
    // pub keywords: Vec<String>,
    pub citekey: String,
}

// INFO & TODO: Iterator needs to process all citekeys (Vec<String>) and should output another vector which holds every single entry as BibiEntry struct (Vec<BibiEntry>). Maybe the BibiEntry struct has to be wrapped inside a larger BibiEntryVec/BibiDataSets struct or similar. -> Iterator for BibiDataSets!

impl BibiEntry {
    pub fn new(citekey: &str) -> Self {
        Self {
            authors: Self::get_authors(citekey),
            title: Self::get_title(citekey),
            year: Self::get_year(citekey),
            pubtype: Self::get_pubtype(citekey),
            // keywords: Self::get_keywords(citekey),
            citekey: citekey.to_string(),
        }
    }

    fn get_authors(citekey: &str) -> String {
        let biblio = BibiMain::new().bibliography;
        let authors = {
            if biblio.get(&citekey).unwrap().author().is_ok() {
                let authors = biblio.get(&citekey).unwrap().author().unwrap();
                if authors.len() > 1 {
                    let authors = format!("{} et al.", authors[0].name);
                    authors
                } else if authors.len() == 1 {
                    let authors = authors[0].name.to_string();
                    authors
                } else {
                    let editors_authors = format!("empty");
                    editors_authors
                }
            } else {
                if biblio.get(&citekey).unwrap().editors().is_ok() {
                    let editors = biblio.get(&citekey).unwrap().editors().unwrap();
                    if editors.len() > 1 {
                        let editors = format!("{} (ed.) et al.", editors[0].0[0].name);
                        editors
                    } else if editors.len() == 1 {
                        let editors = format!("{} (ed.)", editors[0].0[0].name);
                        editors
                    } else {
                        let editors_authors = format!("empty");
                        editors_authors
                    }
                } else {
                    let editors_authors = format!("empty");
                    editors_authors
                }
            }

            // if biblio.get(&citekey).unwrap().editors().is_ok() {
            //     let editors = biblio.get(&citekey).unwrap().editors().unwrap();
            //     if editors.len() > 1 {
            //         let editors = format!("{} (ed.) et al.", editors[0].0[0].name);
            //         editors
            //     } else {
            //         let editors = editors[0].0[0].name.to_string();
            //         editors
            //     }
            // } else {
            //     let editors_authors = format!("empty");
            //     editors_authors
            // }
        };
        authors
    }

    fn get_title(citekey: &str) -> String {
        let biblio = BibiMain::new().bibliography;
        let title = biblio
            .get(&citekey)
            .unwrap()
            .title()
            .unwrap()
            .format_verbatim();
        title
    }

    fn get_year(citekey: &str) -> String {
        let biblio = BibiMain::new().bibliography;
        let year = biblio.get(&citekey).unwrap();
        let year = {
            if year.date().is_ok() {
                let year = year.date().unwrap().to_chunks().format_verbatim();
                let year = year[..4].to_string();
                year
            } else {
                let year = format!("emtpy");
                year
            }
        };
        year

        // let year = &year[..4].to_string();
        // let year = {
        //     if year.is_empty() {
        //         let year = format!("empty");
        //         year
        //     } else {
        //         let year = year.to_string();
        //         year
        //     }
        // };
        // year
    }

    fn get_pubtype(citekey: &str) -> String {
        let biblio = BibiMain::new().bibliography;
        let pubtype = biblio.get(&citekey).unwrap().entry_type.to_string();
        pubtype
    }

    fn get_keywords(citekey: &str) -> String {
        let biblio = BibiMain::new().bibliography;
        let keywords = biblio
            .get(&citekey)
            .unwrap()
            .keywords()
            .unwrap()
            .format_verbatim();
        keywords
    }
}
