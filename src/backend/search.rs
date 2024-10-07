use nucleo_matcher::{
    pattern::{CaseMatching, Normalization, Pattern},
    Config, Matcher,
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct BibiSearch {
    pub search_string: String, // Search string show in footer, used for search
    pub inner_search: bool,    // True, if we trigger a search for already filtered list
    pub filtered_entry_list: Vec<Vec<String>>, // Temporary holds filtered entry list to refilter it
    pub filtered_tag_list: Vec<String>,
}

impl Default for BibiSearch {
    fn default() -> Self {
        Self {
            search_string: String::new(),
            inner_search: false,
            filtered_entry_list: Vec::new(),
            filtered_tag_list: Vec::new(),
        }
    }
}

impl BibiSearch {
    // Stringify inner Vec<String> by joining/concat
    fn convert_to_string(inner_vec: &Vec<String>) -> String {
        inner_vec[0..6].join(" ")
    }

    // Return a filtered entry list
    pub fn search_entry_list(
        search_pattern: &str,
        orig_list: Vec<Vec<String>>,
    ) -> Vec<Vec<String>> {
        // Create a hashmap to connect stingified entry with entry vec
        let mut entry_string_hm: HashMap<String, Vec<String>> = HashMap::new();

        // Convert all entries to string and insert them into the hashmap
        // next to the original inner Vec<String> of the entry list
        for entry in orig_list {
            entry_string_hm.insert(Self::convert_to_string(&entry), entry);
        }

        // Set up matcher (TODO: One time needed only, move to higher level)
        let mut matcher = Matcher::new(Config::DEFAULT);

        // Filter the stringified entries and collect them into a vec
        let filtered_matches: Vec<String> = {
            let matches =
                Pattern::parse(search_pattern, CaseMatching::Ignore, Normalization::Smart)
                    .match_list(entry_string_hm.keys(), &mut matcher);
            matches.into_iter().map(|f| f.0.to_string()).collect()
        };

        // Create filtered entry list and push the inner entry vec's to it
        // Use the filtered stringified hm-key as index
        let mut filtered_list: Vec<Vec<String>> = Vec::new();
        for m in filtered_matches {
            filtered_list.push(entry_string_hm[&m].to_owned());
        }
        filtered_list
    }

    pub fn search_tag_list(search_pattern: &str, orig_list: Vec<String>) -> Vec<String> {
        // Set up matcher (TODO: One time needed only)
        let mut matcher = Matcher::new(Config::DEFAULT);

        // Filter the list items by search pattern
        let filtered_matches: Vec<String> = {
            let matches =
                Pattern::parse(search_pattern, CaseMatching::Ignore, Normalization::Smart)
                    .match_list(orig_list, &mut matcher);
            matches.into_iter().map(|f| f.0.to_string()).collect()
        };
        filtered_matches
    }

    pub fn filter_entries_by_tag(keyword: &str, orig_list: &Vec<Vec<String>>) -> Vec<Vec<String>> {
        let mut filtered_list: Vec<Vec<String>> = Vec::new();

        for e in orig_list {
            if e[4].contains(keyword) {
                filtered_list.push(e.to_owned());
            }
        }

        filtered_list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_join() {
        let bibvec = vec![
            "Author".to_string(),
            "Title".to_string(),
            "1999".to_string(),
            "article".to_string(),
            "hello, bye".to_string(),
            "author_1999".to_string(),
            "An abstract with multiple sentences. Sometimes thats necessary".to_string(),
            "www.bibiman.org".to_string(),
            "/home/file/path.pdf".to_string(),
        ];

        let joined_vec = BibiSearch::convert_to_string(&bibvec);

        assert_eq!(
            joined_vec,
            "Author Title 1999 article hello, bye author_1999"
        )
    }
}
