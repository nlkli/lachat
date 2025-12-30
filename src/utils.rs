use crate::Result;
use std::{io::{self, Read}, os::fd::AsRawFd};

pub fn read_stdin() -> Result<String> {
    let stdin_fd = io::stdin().as_raw_fd();

    if unsafe { libc::isatty(stdin_fd) } == 0 {
        let mut buff = String::new();
        io::stdin().read_to_string(&mut buff)?;
        return Ok(buff);
    }

    Ok(String::default())
}

pub fn fuzzy_search<'a, 'v>(items: &'v [&'a str], query: &str) -> Option<&'a str> {
    if query.len() > 512 {
        return None;
    }
    let query_norm = query
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    let mut best_score = 0.0;
    let mut best_item = None;

    for &item in items {
        let item_norm = item.to_lowercase();

        let jw = strsim::jaro_winkler(&query_norm, &item_norm);
        let lev = strsim::normalized_levenshtein(&query_norm, &item_norm);

        let mut score = 0.7 * jw + 0.3 * lev;

        let query_words: Vec<&str> = query_norm.split(' ').collect();
        let mut word_matches = 0;

        for w in &query_words {
            if item_norm.contains(w) {
                word_matches += 1;
            }
        }

        score += 0.05 * word_matches as f64;

        if score > best_score {
            best_score = score;
            best_item = Some(item);
        }
    }

    best_item
}
