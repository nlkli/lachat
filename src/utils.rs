use crate::{Result, models::laserv::Completion};
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

#[inline]
pub fn is_existing_file(path: &str) -> bool {
    Path::new(path).is_file()
}


#[derive(Debug, Default)]
pub struct LlmResponse {
    pub text: String,
    pub code_blocks: Vec<CodeBlock>,
}

#[derive(Debug)]
pub struct CodeBlock {
    pub language: Option<String>,
    pub code: String,
}

pub fn extract_code_blocks(input: &str) -> LlmResponse {
    let mut result = LlmResponse::default();
    let mut rest = input;

    while let Some(start) = rest.find("```") {
        result.text.push_str(&rest[..start]);

        rest = &rest[start + 3..];

        let (language, after_lang) = match rest.find('\n') {
            Some(nl) => {
                let lang = rest[..nl].trim();
                let lang = if lang.is_empty() {
                    None
                } else {
                    Some(lang.to_string())
                };
                (lang, &rest[nl + 1..])
            }
            None => break,
        };

        if let Some(end) = after_lang.find("```") {
            let code = &after_lang[..end];

            result.code_blocks.push(CodeBlock {
                language,
                code: code.to_string(),
            });

            rest = &after_lang[end + 3..];
        } else {
            result.code_blocks.push(CodeBlock {
                language,
                code: after_lang.to_string(),
            });
            rest = "";
            break;
        }
    }

    result.text.push_str(rest);

    result
}

