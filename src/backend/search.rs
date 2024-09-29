use nucleo_matcher::{
    pattern::{CaseMatching, Normalization, Pattern},
    Config, Matcher,
};
use std::collections::HashMap;

// Stringify inner Vec<String> by joining/concat
fn convert_to_string(inner_vec: &Vec<String>) -> String {
    inner_vec.join(" ")
}

// Return a filtered entry list
pub fn search_entry_list(search_pattern: &str, orig_list: Vec<Vec<String>>) -> Vec<Vec<String>> {
    // Create a hashmap to connect stingified entry with entry vec
    let mut entry_string_hm: HashMap<String, Vec<String>> = HashMap::new();

    // Convert all entries to string and insert them into the hashmap
    // next to the original inner Vec<String> of the entry list
    for entry in orig_list {
        entry_string_hm.insert(convert_to_string(&entry), entry);
    }

    // Set up matcher (TODO: One time needed only, move to higher level)
    let mut matcher = Matcher::new(Config::DEFAULT);

    // Filter the stringified entries and collect them into a vec
    let filtered_matches: Vec<String> = {
        let matches = Pattern::parse(search_pattern, CaseMatching::Ignore, Normalization::Smart)
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
