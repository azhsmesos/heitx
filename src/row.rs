use std::cmp;
use termion::color;
use unicode_segmentation::UnicodeSegmentation;
use crate::SearchDirection;
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
                    res.push_str(&start_highlight);
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
        if after > self.len {
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

    pub fn highlight(&mut self) {
        let mut highlighting = Vec::new();
        for c in self.string.chars() {
            if c.is_ascii_digit() {
                highlighting.push(highlighting::Type::Number);
            } else {
                highlighting.push(highlighting::Type::None);
            }
        }
        self.highlighting = highlighting;
    }
}