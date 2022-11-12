use crate::{Position, Row};
use std::fs;
use std::io::{Error, Write};

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    pub filename: Option<String>,
    dirty: bool,
}

impl Document {
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(filename)?;
        let mut rows = Vec::new();
        for value in contents.lines() {
            rows.push(Row::from(value));
        }
        Ok(Self {
            rows,
            filename: Some(filename.to_string()),
            dirty: false,
        })
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
        if position.y > self.len() {
            return;
        }
        self.dirty = true;
        if c == '\n' {
            self.insert_newline(position);
            return;
        }
        if position.y == self.len() {
            let mut row = Row::default();
            row.insert(0, c);
            self.rows.push(row);
        } else  {
            let row = self.rows.get_mut(position.y).unwrap();
            row.insert(position.x, c);
        }
    }

    // simple delete
    pub fn delete(&mut self, pos: &Position) {
        if pos.y >= self.len() {
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
        if pos.x == self.rows.get_mut(pos.y).unwrap().len() && pos.y < self.len() - 1 {
            let next_row = self.rows.remove(pos.y + 1);
            let row = self.rows.get_mut(pos.y).unwrap();
            row.append(&next_row);
        } else {
            let row = self.rows.get_mut(pos.y).unwrap();
            row.delete(pos.x);
        }
    }

    fn insert_newline(&mut self, pos: &Position) {
        if pos.y == self.len() {
            self.rows.push(Row::default());
            return;
        }

        let new_row = self.rows.get_mut(pos.y).unwrap().split(pos.x);
        self.rows.insert(pos.y + 1, new_row);
    }

    pub fn save_to_disk(&mut self) -> Result<(), Error> {
        if let Some(filename) = &self.filename {
            let mut file = fs::File::create(filename)?;
            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
            self.dirty = true;
        }
        Ok(())
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}