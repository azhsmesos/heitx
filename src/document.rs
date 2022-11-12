use crate::{Position, Row, SearchDirection};
use std::fs;
use std::io::{Error, Write};
use crate::filetype::FileType;

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    pub filename: Option<String>,
    dirty: bool,
    filetype: FileType,
}

impl Document {
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(filename)?;
        let mut rows = Vec::new();
        let filetype = FileType::from(filename);
        for value in contents.lines() {
            let mut row = Row::from(value);
            row.highlight(filetype.highlighting_options(), None);
            rows.push(row);
        }
        Ok(Self {
            rows,
            filename: Some(filename.to_string()),
            dirty: false,
            filetype,
        })
    }

    pub fn filetype(&self) -> String {
        self.filetype.name()
    }

    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    // simple insert
    pub fn insert(&mut self, position: &Position, c: char) {
        if position.y > self.rows.len() {
            return;
        }
        self.dirty = true;
        if c == '\n' {
            self.insert_newline(position);
            return;
        }
        if position.y == self.rows.len() {
            let mut row = Row::default();
            row.insert(0, c);
            row.highlight(self.filetype.highlighting_options(), None);
            self.rows.push(row);
        } else  {
            #[allow(clippy::indexing_slicing)]
            let row = self.rows.get_mut(position.y).unwrap();
            row.insert(position.x, c);
            row.highlight(self.filetype.highlighting_options(), None);
        }
    }

    // simple delete
    #[allow(clippy::integer_arithmetic)]
    pub fn delete(&mut self, pos: &Position) {
        if pos.y >= self.rows.len() {
            return;
        }
        self.dirty = true;
        /*
            What it does is check if we are at the end of a line,
            and if there is a line after that line. If this is the case,
            we remove the next line of vec from our and append it to the
            current line. If this is not the case, we simply t
            ry to delete from the current row.
         */
        if pos.x == self.rows.get_mut(pos.y).unwrap().len() && pos.y + 1 < self.len() {
            let next_row = self.rows.remove(pos.y + 1);
            let row = self.rows.get_mut(pos.y).unwrap();
            row.append(&next_row);
            row.highlight(self.filetype.highlighting_options(), None);
        } else {
            let row = self.rows.get_mut(pos.y).unwrap();
            row.delete(pos.x);
            row.highlight(self.filetype.highlighting_options(), None);
        }
    }

    fn insert_newline(&mut self, pos: &Position) {
        if pos.y > self.rows.len() {
            return;
        }
        if pos.y == self.rows.len() {
            self.rows.push(Row::default());
            return;
        }
        #[allow(clippy::indexing_slicing)]
        let current_row = &mut self.rows[pos.y];
        let mut new_row = current_row.split(pos.x);
        current_row.highlight(self.filetype.highlighting_options(), None);
        new_row.highlight(self.filetype.highlighting_options(), None);
        #[allow(clippy::integer_arithmetic)]
        self.rows.insert(pos.y + 1, new_row);
    }

    pub fn save_to_disk(&mut self) -> Result<(), Error> {
        if let Some(filename) = &self.filename {
            let mut file = fs::File::create(filename)?;
            self.filetype = FileType::from(filename);
            for row in &mut self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
                row.highlight(self.filetype.highlighting_options(), None)
            }
            self.dirty = false;
        }
        Ok(())
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn search(&self, query: &str, after: &Position, direction: SearchDirection) -> Option<Position> {
        if after.y >= self.rows.len() {
            return None;
        }
        let mut position = Position { x: after.x, y: after.y };
        let start = if direction == SearchDirection::Forward {
            after.y
        } else {
            0
        };
        let end = if direction == SearchDirection::Forward {
            self.rows.len()
        } else {
            after.y.saturating_add(1)
        };
        for _ in start..end {
            if let Some(row) = self.rows.get(position.y) {
                if let Some(x) = row.search(&query, position.x, direction) {
                    position.x = x;
                    return Some(position);
                }
                if direction == SearchDirection::Forward {
                    position.y = position.y.saturating_add(1);
                    position.x = 0;
                } else {
                    position.y = position.y.saturating_sub(1);
                    position.x = self.rows[position.y].len();
                }
            } else {
                return None;
            }
        }
        None
    }

    pub fn highlight(&mut self, word: Option<&str>) {
        for row in &mut self.rows {
            row.highlight(self.filetype.highlighting_options(), word);
        }
    }
}