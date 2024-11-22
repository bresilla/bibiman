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

use biblatex::{self, Bibliography};
use biblatex::{ChunksExt, Type};
use color_eyre::owo_colors::OwoColorize;
use itertools::Itertools;
use std::ffi::OsString;
use std::{fs, path::PathBuf};

use crate::cliargs;

// Set necessary fields
// TODO: can surely be made more efficient/simpler
#[derive(Debug)]
pub struct BibiSetup {
    // pub bibfile: PathBuf,           // path to bibfile
    pub bibfilestring: String,      // content of bibfile as string
    pub bibliography: Bibliography, // parsed bibliography
    pub citekeys: Vec<String>,      // list of all citekeys
    pub keyword_list: Vec<String>,  // list of all available keywords
    pub entry_list: Vec<BibiData>,  // List of all entries
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BibiData {
    pub authors: String,
    pub title: String,
    pub year: String,
    pub pubtype: String,
    pub keywords: String,
    pub citekey: String,
    pub abstract_text: String,
    pub doi_url: Option<String>,
    pub filepath: Option<OsString>,
    pub subtitle: Option<String>,
}

impl BibiSetup {
    pub fn new(main_bibfiles: &[PathBuf]) -> Self {
        // TODO: Needs check for config file path as soon as config file is impl
        Self::check_files(main_bibfiles);
        let bibfilestring = Self::bibfiles_to_string(main_bibfiles);
        let bibliography = biblatex::Bibliography::parse(&bibfilestring).unwrap();
        let citekeys = Self::get_citekeys(&bibliography);
        let keyword_list = Self::collect_tag_list(&citekeys, &bibliography);
        let entry_list = Self::create_entry_list(&citekeys, &bibliography);
        Self {
            // bibfile,
            bibfilestring,
            bibliography,
            citekeys,
            keyword_list,
            entry_list,
        }
    }

    // Check which file format the passed file has
    fn check_files(main_bibfiles: &[PathBuf]) {
        if main_bibfiles.is_empty() {
            println!(
                "{}",
                "No bibfile passed as argument. Please select a valid file."
                    .red()
                    .bold()
            );
            println!();
            println!("{}", cliargs::help_func());
            std::process::exit(1)
        } else {
            // Loop over all files and check for the correct extension
            main_bibfiles.iter().for_each(|f| if f.extension().is_some() && f.extension().unwrap() != "bib" {
                panic!("File \'{}\' has no valid extension. Please select a file with \'.bib\' extension", f.to_str().unwrap())
            });
        }
    }

    fn bibfiles_to_string(main_bibfiles: &[PathBuf]) -> String {
        // Map PathBufs to String anc join the vector into one big string
        // This behaviour is needed by the biblatex crate for parsing
        let file_strings: Vec<String> = main_bibfiles
            .iter()
            .map(|f| fs::read_to_string(f).unwrap())
            .collect();

        file_strings.join("\n")
    }

    fn create_entry_list(citekeys: &[String], bibliography: &Bibliography) -> Vec<BibiData> {
        citekeys
            .iter()
            .map(|k| BibiData {
                authors: Self::get_authors(k, bibliography),
                title: Self::get_title(k, bibliography),
                year: Self::get_year(k, bibliography),
                pubtype: Self::get_pubtype(k, bibliography),
                keywords: Self::get_keywords(k, bibliography),
                citekey: k.to_owned(),
                abstract_text: Self::get_abstract(k, bibliography),
                doi_url: Self::get_weblink(k, bibliography),
                filepath: Self::get_filepath(k, bibliography),
                subtitle: Self::get_subtitle(k, bibliography),
            })
            .collect()
    }

    // get list of citekeys from the given bibfile
    // this list is the base for further operations on the bibentries
    // since it is the entry point of the biblatex crate.
    pub fn get_citekeys(bibstring: &Bibliography) -> Vec<String> {
        let citekeys: Vec<String> = bibstring.keys().map(|k| k.to_owned()).collect();
        citekeys
    }

    // collect all keywords present in the bibliography
    // sort them and remove duplicates
    // this list is for fast filtering entries by topics/keyowrds
    pub fn collect_tag_list(citekeys: &[String], biblio: &Bibliography) -> Vec<String> {
        // Initialize vector collecting all keywords
        let mut keyword_list = vec![];

        // Loop over entries and collect all keywords
        for i in citekeys {
            if biblio.get(i).unwrap().keywords().is_ok() {
                let items = biblio.get(i).unwrap().keywords().unwrap().format_verbatim();
                // Split keyword string into slices, trim leading and trailing
                // whitespaces, remove empty slices, and collect them
                let mut key_vec: Vec<String> = items
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                // Append keywords to vector
                keyword_list.append(&mut key_vec);
            }
        }

        // Sort the vector and remove duplicates
        // keyword_list.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        keyword_list.sort_by_key(|a| a.to_lowercase());
        keyword_list.dedup();
        keyword_list
    }

    pub fn get_authors(citekey: &str, biblio: &Bibliography) -> String {
        if biblio.get(citekey).unwrap().author().is_ok() {
            let authors = biblio.get(citekey).unwrap().author().unwrap();
            if authors.len() > 1 {
                authors.iter().map(|a| &a.name).join(", ")
            } else if authors.len() == 1 {
                authors[0].name.to_string()
            } else {
                "empty".to_string()
            }
        } else if !biblio.get(citekey).unwrap().editors().unwrap().is_empty() {
            let editors = biblio.get(citekey).unwrap().editors().unwrap();
            if editors[0].0.len() > 1 {
                format!("{} (ed.)", editors[0].0.iter().map(|e| &e.name).join(", "))
            } else if editors[0].0.len() == 1 {
                format!("{} (ed.)", editors[0].0[0].name)
            } else {
                "empty".to_string()
            }
        } else {
            "empty".to_string()
        }
    }

    pub fn get_title(citekey: &str, biblio: &Bibliography) -> String {
        if biblio.get(citekey).unwrap().title().is_ok() {
            biblio
                .get(citekey)
                .unwrap()
                .title()
                .unwrap()
                .format_verbatim()
        } else {
            "no title".to_string()
        }
    }

    pub fn get_year(citekey: &str, biblio: &Bibliography) -> String {
        let bib = biblio.get(citekey).unwrap();
        if bib.date().is_ok() {
            let year = bib.date().unwrap().to_chunks().format_verbatim();
            year[..4].to_string()
        } else {
            "n.d.".to_string()
        }
    }

    pub fn get_pubtype(citekey: &str, biblio: &Bibliography) -> String {
        biblio.get(citekey).unwrap().entry_type.to_string()
    }

    pub fn get_keywords(citekey: &str, biblio: &Bibliography) -> String {
        if biblio.get(citekey).unwrap().keywords().is_ok() {
            biblio
                .get(citekey)
                .unwrap()
                .keywords()
                .unwrap()
                .format_verbatim()
        } else {
            "".to_string()
        }
    }

    pub fn get_abstract(citekey: &str, biblio: &Bibliography) -> String {
        if biblio.get(citekey).unwrap().abstract_().is_ok() {
            biblio
                .get(citekey)
                .unwrap()
                .abstract_()
                .unwrap()
                .format_verbatim()
        } else {
            "no abstract".to_string()
        }
    }

    pub fn get_weblink(citekey: &str, biblio: &Bibliography) -> Option<String> {
        let bib = biblio.get(citekey).unwrap();
        if bib.doi().is_ok() {
            Some(bib.doi().unwrap())
        } else if bib.url().is_ok() {
            Some(bib.url().unwrap())
        } else {
            None
        }
    }

    pub fn get_filepath(citekey: &str, biblio: &Bibliography) -> Option<OsString> {
        if biblio.get(citekey).unwrap().file().is_ok() {
            Some(biblio.get(citekey).unwrap().file().unwrap().into())
        } else {
            None
        }
    }

    pub fn get_subtitle(citekey: &str, biblio: &Bibliography) -> Option<String> {
        if biblio.get(citekey).unwrap().subtitle().is_ok() {
            Some(
                biblio
                    .get(citekey)
                    .unwrap()
                    .subtitle()
                    .unwrap()
                    .format_verbatim(),
            )
        } else {
            None
        }
    }
}
