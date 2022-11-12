
use std::cmp;
use termion::color;
use unicode_segmentation::UnicodeSegmentation;
use crate::{HighlightingOptions, SearchDirection};
use crate::highlighting;

#[derive(Default)]
pub struct Row {
    string: String,
    highlighting: Vec<highlighting::Type>,
    len: usize,
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        Self {
            string: String::from(slice),
            highlighting: Vec::new(),
            len: slice.graphemes(true).count(),
        }
    }
}

impl Row {
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);
        let mut res = String::new();
        let mut current_highlighting = &highlighting::Type::None;
        #[allow(clippy::integer_arithmetic)]
        for (index, grapheme) in self.string[..]
            .graphemes(true)
            .enumerate()
            .skip(start)
            .take(end - start) {
            if let Some(c) = grapheme.chars().next() {
                let highlighting_type = self.highlighting.get(index).unwrap_or(&highlighting::Type::None);
                if highlighting_type != current_highlighting {
                    current_highlighting = highlighting_type;
                    let start_highlight = format!("{}", termion::color::Fg(highlighting_type.to_color()));
                    res.push_str(&start_highlight[..]);
                }
                if c == '\t' {
                    res.push_str(" ");
                } else {
                    res.push(c);
                }
            }
        }
        let end_highlight = format!("{}", termion::color::Fg(color::Reset));
        res.push_str(&end_highlight[..]);
        res
    }

    pub fn len(&self) -> usize {
       self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn insert(&mut self, pos: usize, c: char) {
        if pos >= self.len {
            self.string.push(c);
            self.len += 1;
            return;
        }
        let mut res: String = String::new();
        let mut len = 0;
        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            len += 1;
            if index == pos {
                len += 1;
                res.push(c);
            }
            res.push_str(grapheme);
        }
        self.len = len;
        self.string = res;
    }

    #[allow(clippy::integer_arithmetic)]
    pub fn delete(&mut self, pos: usize) {
        if pos >= self.len {
            return;
        }
        let mut res: String = String::new();
        let mut len = 0;
        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            len += 1;
            if index != pos {
                len += 1;
                res.push_str(grapheme);
            }
        }
        self.len = len;
        self.string = res;
    }

    pub fn append(&mut self, new: &Self) {
        self.string = format!("{}{}", self.string, new.string);
        self.len += new.len;
    }

    pub fn split(&mut self, pos: usize) -> Self {
        let mut row: String = String::new();
        let mut len = 0;
        let mut split_row: String = String::new();
        let mut split_len = 0;
        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            if index < pos {
                len += 1;
                row.push_str(grapheme);
            } else {
                split_len += 1;
                split_row.push_str(grapheme);
            }
        }
        self.string = row;
        self.len = len;
        Self {
            string: split_row,
            highlighting: Vec::new(),
            len: split_len,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.string.as_bytes()
    }

    pub fn search(&self, query: &str, after: usize, direction: SearchDirection) -> Option<usize> {
        if after > self.len || query.is_empty() {
            return None;
        }
        let start = if direction == SearchDirection::Forward {
            after
        } else {
            0
        };
        let end = if direction == SearchDirection::Forward {
            self.len
        } else {
            after
        };
        let substring: String = self.string[..]
            .graphemes(true)
            .skip(start)
            .take(end - start)
            .collect();
        let matching_byte_index = if direction == SearchDirection::Forward {
            substring.find(query)
        } else {
            substring.rfind(query)
        };
        if let Some(matching_byte_index) = matching_byte_index {
           for (grapheme_index, (byte_index, _)) in substring[..].grapheme_indices(true).enumerate() {
               if matching_byte_index == byte_index {
                   return Some(grapheme_index + start);
               }
           }
        }
        None
    }

    fn highlight_match(&mut self, word: Option<&str>) {
        if let Some(word) = word {
            if word.is_empty() {
                return;
            }
            let mut index = 0;
            while let Some(search_match) = self.search(word, index, SearchDirection::Forward) {
                if let Some(next_index) = search_match.checked_add(word[..].graphemes(true).count()) {
                    #[allow(clippy::indexing_slicing)]
                    for i in index.saturating_add(search_match)..next_index {
                        self.highlighting[i] = highlighting::Type::Match;
                    }
                    index = next_index;
                } else {
                    break;
                }
            }
        }
    }

    fn highlight_char(&mut self, index: &mut usize, opts: &HighlightingOptions, c: char, chars: &[char]) -> bool {
        if opts.characters() && c == '\'' {
            if let Some(next_char) = chars.get(index.saturating_add(1)) {
                let closing_index = if *next_char == '\\' {
                    index.saturating_add(3)
                } else {
                    index.saturating_add(2)
                };
                if let Some(closing_char) = chars.get(closing_index) {
                    if *closing_char == '\'' {
                        for _ in 0..=closing_index.saturating_sub(*index) {
                            self.highlighting.push(highlighting::Type::Character);
                            *index += 1;
                        }
                        return true;
                    }
                }
            }
        }
        false
    }

    fn highlight_comment(&mut self, index: &mut usize, opts: &HighlightingOptions, c: char, chars: &[char]) -> bool {
        if opts.comments() && c == '/' && *index < chars.len() {
            if let Some(next_char) = chars.get(index.saturating_add(1)) {
                if *next_char == '/' {
                    for _ in *index..chars.len() {
                        self.highlighting.push(highlighting::Type::Comment);
                        *index += 1;
                    }
                    return true;
                }
            }
        }
        false
    }

    fn highlight_string(&mut self, index: &mut usize, opts: &HighlightingOptions, c: char, chars: &[char]) -> bool {
        if opts.strings() && c == '"' {
            loop {
                self.highlighting.push(highlighting::Type::String);
                *index += 1;
                if let Some(next_char) = chars.get(*index) {
                    if *next_char == '"' {
                        break;
                    }
                } else {
                    break;
                }
            }
            self.highlighting.push(highlighting::Type::String);
            *index += 1;
            return true;
        }
        false
    }

    fn highlight_number(&mut self, index: &mut usize, opts: &HighlightingOptions, c: char, chars: &[char]) -> bool {
        if opts.numbers() && c.is_ascii_digit() {
            if *index > 0 {
                #[allow(clippy::indexing_slicing, clippy::integer_arithmetic)]
                let prev_char = chars[*index - 1];
                if !is_separators(prev_char) {
                    return false;
                }
            }
            loop {
                self.highlighting.push(highlighting::Type::Number);
                *index += 1;
                if let Some(next_char) = chars.get(*index) {
                    if *next_char != '.' && !next_char.is_ascii_digit() {
                        break;
                    }
                } else {
                    break;
                }
            }
            return true;
        }
        false
    }

    fn highlight_str(&mut self, index: &mut usize, substring: &str, chars: &[char], hl_type: highlighting::Type) -> bool {
        if substring.is_empty() {
            return false;
        }
        for (substring_index, c) in substring.chars().enumerate() {
            if let Some(next_char) = chars.get(index.saturating_add(substring_index)) {
                if *next_char != c {
                    return false;
                }
            } else {
                return false;
            }
        }
        for _ in 0..substring.len() {
            self.highlighting.push(hl_type);
            *index += 1;
        }
        true
    }

    fn highlight_keywords(&mut self, index: &mut usize, chars: &[char], keywords: &[String], hl_type: highlighting::Type) -> bool {
        if *index > 0 {
            #[allow(clippy::indexing_slicing, clippy::integer_arithmetic)]
                let prev_char = chars[*index - 1];
            if !is_separators(prev_char) {
                return false;
            }
        }

        for word in keywords {
            if *index < chars.len().saturating_sub(word.len()) {
                #[allow(clippy::indexing_slicing, clippy::integer_arithmetic)]
                    let next_char = chars[*index + word.len()];
                if !is_separators(next_char) {
                    continue;
                }
            }
            if self.highlight_str(index, &word, chars, hl_type) {
                return true;
            }
        }
        false
    }

    fn highlight_primary_keywords(&mut self, index: &mut usize, opts: &HighlightingOptions, chars: &[char]) -> bool {
        self.highlight_keywords(index, chars, opts.primary_keywords(), highlighting::Type::PrimaryKeywords)
    }

    fn highlight_secondary_keywords(&mut self, index: &mut usize, opts: &HighlightingOptions, chars: &[char]) -> bool {
        self.highlight_keywords(index, chars, opts.secondary_keywords(), highlighting::Type::SecondaryKeywords)
    }

    pub fn highlight(&mut self, opts: &HighlightingOptions, word: Option<&str>) {
        self.highlighting = Vec::new();
        let chars: Vec<char> = self.string.chars().collect();
        let mut index = 0;
        while let Some(c) = chars.get(index) {
            if self.highlight_char(&mut index, opts, *c, &chars)
                || self.highlight_comment(&mut index, opts, *c, &chars)
                || self.highlight_string(&mut index, opts, *c, &chars)
                || self.highlight_number(&mut index, opts, *c, &chars)
                || self.highlight_primary_keywords(&mut index, &opts, &chars)
                || self.highlight_secondary_keywords(&mut index, &opts, &chars){
                continue;
            }
            self.highlighting.push(highlighting::Type::None);
            index += 1;
        }
        self.highlight_match(word);
    }
}

fn is_separators(c: char) -> bool {
    c.is_ascii_punctuation() || c.is_ascii_whitespace()
}