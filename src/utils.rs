use crate::Result;
use std::process::Command;
use std::{
    io::{self, Read, Write},
    os::fd::AsRawFd,
    path::Path,
};

pub fn open_url(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(url).spawn()?;
    }

    Ok(())
}

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

pub struct DualWriter<W1, W2> {
    pub w1: W1,
    pub w2: W2,
}

impl<W1: Write, W2: Write> Write for DualWriter<W1, W2> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n1 = self.w1.write(buf)?;
        let n2 = self.w2.write(buf)?;
        Ok(n1.min(n2))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.w1.flush()?;
        self.w2.flush()?;
        Ok(())
    }
}

pub fn extend_args(args: &[String], from: &[&str]) -> Vec<String> {
    let mut result = args.to_vec();
    for i in (0..from.len()).step_by(2) {
        let name = from[i];
        let value = from.get(i + 1).map_or("", |v| *v);
        if let Some(pos) = args.iter().position(|v| v == name) {
            result.get_mut(pos + 1).map(|v| *v = value.into());
        } else {
            result.push(name.into());
            result.push(value.into());
        }
    }
    result
}
