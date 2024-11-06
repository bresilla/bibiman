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
use itertools::Itertools;
use std::{fs, path::PathBuf};

#[derive(Debug)]
pub enum FileFormat {
    BibLatex,
    Hayagriva,
}

// Set necessary fields
// TODO: can surely be made more efficient/simpler
#[derive(Debug)]
pub struct BibiSetup {
    // pub bibfile: PathBuf,           // path to bibfile
    pub bibfile_format: FileFormat, // Format of passed file
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
    pub doi_url: String,
    pub filepath: String,
    pub subtitle: Option<String>,
}

impl BibiSetup {
    pub fn new(main_bibfile: &PathBuf) -> Self {
        // TODO: Needs check for config file path as soon as config file is impl
        let bibfile_format = Self::check_file_format(main_bibfile);
        let bibfilestring = fs::read_to_string(main_bibfile).unwrap();
        let bibliography = biblatex::Bibliography::parse(&bibfilestring).unwrap();
        let citekeys = Self::get_citekeys(&bibliography);
        let keyword_list = Self::collect_tag_list(&citekeys, &bibliography);
        let entry_list = Self::create_entry_list(&citekeys, &bibliography);
        Self {
            // bibfile,
            bibfile_format,
            bibfilestring,
            bibliography,
            citekeys,
            keyword_list,
            entry_list,
        }
    }

    // Check which file format the passed file has
    fn check_file_format(main_bibfile: &PathBuf) -> FileFormat {
        let extension = main_bibfile.extension().unwrap().to_str();

        match extension {
            Some("yml") => FileFormat::Hayagriva,
            Some("yaml") => FileFormat::Hayagriva,
            Some("bib") => FileFormat::BibLatex,
            Some(_) => panic!("The extension {:?} is no valid bibfile", extension.unwrap()),
            None => panic!("The given path {:?} holds no valid file", main_bibfile),
        }
    }

    fn create_entry_list(citekeys: &[String], bibliography: &Bibliography) -> Vec<BibiData> {
        citekeys
            .into_iter()
            .map(|k| BibiData {
                authors: Self::get_authors(&k, &bibliography),
                title: Self::get_title(&k, &bibliography),
                year: Self::get_year(&k, &bibliography),
                pubtype: Self::get_pubtype(&k, &bibliography),
                keywords: Self::get_keywords(&k, &bibliography),
                citekey: k.to_owned(),
                abstract_text: Self::get_abstract(&k, &bibliography),
                doi_url: Self::get_weblink(&k, &bibliography),
                filepath: Self::get_filepath(&k, &bibliography),
                subtitle: Self::get_subtitle(&k, &bibliography),
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
            if biblio.get(&i).unwrap().keywords().is_ok() {
                let items = biblio
                    .get(&i)
                    .unwrap()
                    .keywords()
                    .unwrap()
                    .format_verbatim();
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
        keyword_list.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        keyword_list.dedup();
        keyword_list
    }

    pub fn get_authors(citekey: &str, biblio: &Bibliography) -> String {
        if biblio.get(&citekey).unwrap().author().is_ok() {
            let authors = biblio.get(&citekey).unwrap().author().unwrap();
            if authors.len() > 1 {
                let all_authors = authors.iter().map(|a| &a.name).join(", ");
                all_authors
            } else if authors.len() == 1 {
                let authors = authors[0].name.to_string();
                authors
            } else {
                let editors_authors = format!("empty");
                editors_authors
            }
        } else {
            if !biblio.get(&citekey).unwrap().editors().unwrap().is_empty() {
                let editors = biblio.get(&citekey).unwrap().editors().unwrap();
                if editors[0].0.len() > 1 {
                    // let editors = format!("{} (ed.) et al.", editors[0].0[0].name);
                    let mut editors = editors[0].0.iter().map(|e| &e.name).join(", ");
                    editors.push_str(" (ed.)");
                    editors
                } else if editors[0].0.len() == 1 {
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
        let keywords = {
            if biblio.get(&citekey).unwrap().keywords().is_ok() {
                let keywords = biblio
                    .get(&citekey)
                    .unwrap()
                    .keywords()
                    .unwrap()
                    .format_verbatim();
                keywords
            } else {
                let keywords = String::from("");
                keywords
            }
        };
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

    pub fn get_weblink(citekey: &str, biblio: &Bibliography) -> String {
        if let true = biblio.get(&citekey).unwrap().doi().is_ok() {
            let url = biblio.get(&citekey).unwrap().doi().unwrap();
            url
        } else if let true = biblio.get(&citekey).unwrap().url().is_ok() {
            let url = biblio.get(&citekey).unwrap().url().unwrap();
            url
        } else {
            let url = "".to_string();
            url
        }
    }

    pub fn get_filepath(citekey: &str, biblio: &Bibliography) -> String {
        if let true = biblio.get(&citekey).unwrap().file().is_ok() {
            let file = biblio.get(&citekey).unwrap().file().unwrap();
            file
        } else {
            let file = "".to_string();
            file
        }
    }

    pub fn get_subtitle(citekey: &str, biblio: &Bibliography) -> Option<String> {
        if biblio.get(&citekey).unwrap().subtitle().is_ok() {
            Some(
                biblio
                    .get(&citekey)
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
