use biblatex::Bibliography;
use regex::Regex;
use std::{
    fs,
    path::{Path, PathBuf},
};

use super::cliargs::{CLIArgs, PosArgs};

// Set necessary fields
// TODO: can surely be made more efficient/simpler
pub struct Bibi {
    pub citekeys: Vec<String>,
    // pub bibliography: Bibliography,
}

pub fn get_bibfile(filename: impl AsRef<Path>) -> String {
    let bibfile = fs::read_to_string(&filename).unwrap();
    bibfile
}

pub fn get_citekeys(bibstring: &Bibliography) -> Vec<String> {
    // let bib = Bibliography::parse(&get_bibfile(CLIArgs::parse_cli_args().bibfilearg)).unwrap();
    // // Define Regex to match citekeys
    // let re = Regex::new(r"(?m)^\@[a-zA-Z]*\{(.*)\,").unwrap();
    // // Declare empty vector to fill with captured keys
    // // Has to be Vec<&str> because of captures_iter method
    // let mut keys = vec![];
    // for (_, [key]) in re.captures_iter(&bibfilestring).map(|c| c.extract()) {
    //     keys.push(key);
    // }
    // // Transform Vec<&str> to Vec<String> which is needed by the struct Bibi
    // let mut citekeys: Vec<String> = keys.into_iter().map(String::from).collect();
    // // Sort vector items case-insensitive
    // citekeys.sort_by_key(|name| name.to_lowercase());
    // citekeys
    let mut citekeys: Vec<String> = bibstring.iter().map(|entry| entry.to_owned().key).collect();
    citekeys.sort_by_key(|name| name.to_lowercase());
    citekeys
}

impl Bibi {
    pub fn new() -> Self {
        // TODO: Needs check for config file path as soon as config file is impl
        let bib = Bibliography::parse(&get_bibfile(PosArgs::parse_pos_args().bibfilearg)).unwrap();
        Self {
            citekeys: get_citekeys(&bib),
            // bibliography: biblatex::Bibliography::parse(&bibfilestring).unwrap(),
        }
    }
}
