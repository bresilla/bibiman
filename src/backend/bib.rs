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

use biblatex::{self, Bibliography};
use biblatex::{ChunksExt, Type};

// Set necessary fields
// TODO: can surely be made more efficient/simpler
#[derive(Debug)]
pub struct BibiMain {
    pub bibfile: PathBuf,           // path to bibfile
    pub bibfilestring: String,      // content of bibfile as string
    pub bibliography: Bibliography, // parsed bibliography
    pub citekeys: Vec<String>,      // list of all citekeys
                                    // pub bibentries: BibiDataSets,
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

#[derive(Debug)]
pub struct BibiData {
    pub entry_list: BibiDataSets,
}

impl BibiData {
    pub fn new(biblio: &Bibliography, citekeys: &Vec<String>) -> Self {
        Self {
            entry_list: {
                let bibentries = citekeys
                    .into_iter()
                    .map(|citekey| BibiEntry::new(&citekey, &biblio))
                    .collect();
                BibiDataSets { bibentries }
            },
        }
    }
}

// Parent struct which keeps the Vector of all bibentries
// Necessary for implementing FromIterator
#[derive(Debug)]
pub struct BibiDataSets {
    pub bibentries: Vec<Vec<String>>,
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

impl BibiEntry {
    pub fn new(citekey: &str, biblio: &Bibliography) -> Vec<String> {
        vec![
            Self::get_authors(&citekey, &biblio),
            Self::get_title(&citekey, &biblio),
            Self::get_year(&citekey, &biblio),
            Self::get_pubtype(&citekey, &biblio),
            citekey.to_string(),
        ]
    }

    pub fn get_authors(citekey: &str, biblio: &Bibliography) -> String {
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
        };
        authors
    }

    pub fn get_title(citekey: &str, biblio: &Bibliography) -> String {
        let title = {
            if biblio.get(&citekey).unwrap().title().is_ok() {
                let title = biblio
                    .get(&citekey)
                    .unwrap()
                    .title()
                    .unwrap()
                    .format_verbatim();
                title
            } else {
                let title = format!("no title");
                title
            }
        };
        title
    }

    pub fn get_year(citekey: &str, biblio: &Bibliography) -> String {
        let year = biblio.get(&citekey).unwrap();
        let year = {
            if year.date().is_ok() {
                let year = year.date().unwrap().to_chunks().format_verbatim();
                let year = year[..4].to_string();
                year
            } else {
                let year = format!("n.d.");
                year
            }
        };
        year
    }

    pub fn get_pubtype(citekey: &str, biblio: &Bibliography) -> String {
        let pubtype = biblio.get(&citekey).unwrap().entry_type.to_string();
        pubtype
    }

    pub fn get_keywords(citekey: &str, biblio: &Bibliography) -> String {
        let keywords = biblio
            .get(&citekey)
            .unwrap()
            .keywords()
            .unwrap()
            .format_verbatim();
        keywords
    }

    pub fn get_abstract(citekey: &str, biblio: &Bibliography) -> String {
        let text = {
            if biblio.get(&citekey).unwrap().abstract_().is_ok() {
                let abstract_text = biblio
                    .get(&citekey)
                    .unwrap()
                    .abstract_()
                    .unwrap()
                    .format_verbatim();
                abstract_text
            } else {
                let abstract_text = format!("No abstract");
                abstract_text
            }
        };
        text
    }
}
