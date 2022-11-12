use std::env;
use std::time::{Duration, Instant};
use crate::{Document, Row, Terminal};
use termion::event::Key;
use termion::color;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const STATUS_FG_COLOR: color::LightBlack = color::LightBlack;
const QUIT_COUNT: u8 = 2;

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    document: Document,
    offset: Position,
    status_message: StatusMessage,
    quit_count: u8,
}

#[derive(PartialEq, Copy, Clone)]
pub enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Default, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

struct StatusMessage {
    text: String,
    time: Instant,
}

impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            time: Instant::now(),
            text: message,
        }
    }
}

impl Editor {
    pub fn run(&mut self) {
        loop {
            if let Err(error) = self.refresh_screen() {
                close(&error);
            }
            if self.should_quit {
                break;
            }
            if let Err(error) = self.process_key() {
                close(&error);
            }
        }
    }

    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status = String::from("HELP: Ctrl-c = quit | Ctrl-s = save | Ctrl-f = search");
        let document = if let Some(filename) = args.get(1) {
            let doc = Document::open(filename);
            if let Ok(doc) = doc{
                doc
            } else {
                initial_status = format!("ERR: could not open file {}", filename);
                Document::default()
            }
        } else {
            Document::default()
        };
        Self {
            should_quit: false,
            terminal: Terminal::default().expect("failed to initialize heitx terminal"),
            cursor_position: Position::default(),
            document,
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),
            quit_count: QUIT_COUNT,
        }
    }

    fn process_key(&mut self) -> Result<(), std::io::Error> {
        let press = Terminal::read_key()?;
        match press {
            Key::Ctrl('c') => {
                if self.quit_count > 0 && self.document.is_dirty() {
                    self.status_message = StatusMessage::from(
                    format!("WARING! file has unsaved changes, press Ctrl-s {} more count to quit.", self.quit_count));
                    self.quit_count -= 1;
                    return Ok(());
                }
                self.should_quit = true
            },
            Key::Ctrl('s') => self.save(),
            Key::Ctrl('f') => self.search(),
            Key::Char(c) => {
                self.document.insert(&self.cursor_position, c);
                self.move_cursor(Key::Right);
            },
            Key::Delete => self.document.delete(&self.cursor_position),
            Key::Backspace => {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    self.move_cursor(Key::Left);
                    self.document.delete(&self.cursor_position);
                }
            }
            Key::Up
            | Key::Down
            | Key::Left
            | Key::Right
            | Key::PageDown
            | Key::PageUp
            | Key::End
            | Key::Home => self.move_cursor(press),
            _ => (),
        }
        self.scroll();
        if self.quit_count < QUIT_COUNT {
            self.quit_count = QUIT_COUNT;
            self.status_message = StatusMessage::from(String::new());
        }
        Ok(())
    }

    fn save(&mut self) {
        if self.document.filename.is_none() {
            let new_filename = self.prompt("save as: ", |_, _, _| {}).unwrap_or(None);
            if new_filename.is_none() {
                self.status_message = StatusMessage::from("save aborted.".to_string());
                return;
            }
            self.document.filename = new_filename;
        }
        if self.document.save_to_disk().is_ok() {
            self.status_message = StatusMessage::from("file saved to successfully.".to_string());
        } else {
            self.status_message = StatusMessage::from("error writing to file!".to_string());
        }
    }

    fn prompt<C>(&mut self, prompt: &str, mut callback: C) -> Result<Option<String>, std::io::Error> where C: FnMut(&mut Self, Key, &String), {
        let mut res = String::new();
        loop {
            self.status_message = StatusMessage::from(format!("{}{}", prompt, res));
            self.refresh_screen()?;
            let key = Terminal::read_key()?;
            match key {
                Key::Backspace => {
                    if !res.is_empty() {
                        res.truncate(res.len().saturating_sub(1))
                    }
                }
                Key::Char('\n') => break,
                Key::Char(c) => {
                    if !c.is_control() {
                        res.push(c);
                    }
                },
                Key::Esc => {
                    res.truncate(0);
                    break;
                }
                _ => (),
            }
            callback(self, key, &res);
        }
        self.status_message = StatusMessage::from(String::new());
        if res.is_empty() {
            return Ok(None);
        }
        Ok(Some(res))
    }

    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position::default());
        if self.should_quit {
            Terminal::clear_screen();
            println!("heitx terminal exit...\r");
        } else {
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            Terminal::cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }
        Terminal::cursor_show();
        Terminal::flush()
    }

    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        let message = &self.status_message;
        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            print!("{}", text);
        }
    }

    fn draw_status_bar(&self) {
        let mut status;
        let width = self.terminal.size().width as usize;
        let mod_indicator = if self.document.is_dirty() {
            "(modified)"
        } else {
            ""
        };
        let mut filename = "[No Name]".to_string();
        if let Some(name) = &self.document.filename {
            filename = name.clone();
            filename.truncate(20);
        }
        status = format!("{} - {} lines{}", filename, self.document.len(), mod_indicator);
        let line_indict = format!("{}/{}", self.cursor_position.y.saturating_add(1), self.document.len());
        let len = status.len() + line_indict.len();
        if width > len {
            status.push_str(&" ".repeat(width.saturating_sub(len)));
        }
        status = format!("{}{}", status, line_indict);
        status.truncate(width);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        println!("{}\r", status);
        Terminal::reset_fg_color();
        Terminal::reset_bg_color();
    }

    fn draw_rows(&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0..height {
            Terminal::clear_current_line();
            if let Some(row) = self.document.row(self.offset.y.saturating_add(terminal_row as usize)) {
                self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_info();
            } else {
                println!("~\r");
            };
        }
    }

    pub fn draw_row(&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x.saturating_add(width);
        let row = row.render(start, end);
        println!("{}\r", row);
    }

    fn move_cursor(&mut self, key: Key) {
        let terminal_height = self.terminal.size().height as usize;
        let Position { mut y, mut x } = self.cursor_position;
        let height = self.document.len();
        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        match key {
            Key::Up => y = y.saturating_sub(1),
            Key::Down => {
                if y < height {
                    y = y.saturating_add(1);
                }
            },
            // moving left at the start of a line
            Key::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    if let Some(row) = self.document.row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    }
                }
            },
            // moving right at the end of a line
            Key::Right => {
                if x < width {
                    x += 1;
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            },
            // scrolling with pageUp and pageDown
            Key::PageUp => {
                y = if y > terminal_height {
                    y.saturating_sub(terminal_height)
                } else {
                    0
                }
            },
            Key::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y.saturating_add(terminal_height)
                } else {
                    height
                }
            },
            Key::Home => x = 0,
            Key::End => x = width,
            _ => (),
        }
        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        if x > width {
            x = width;
        }
        self.cursor_position = Position { x, y };
    }

    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let mut offset = &mut self.offset;
        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }
        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }

    fn draw_welcome_info(&self) {
        let mut welcome_message = format!("heitx editor --version {}", VERSION);
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        #[allow(clippy::integer_arithmetic, clippy::integer_division)]
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }

    fn search(&mut self) {
        let old_position = self.cursor_position.clone();
        let mut direction = SearchDirection::Forward;
        let query = self.prompt("Search(ESC to cancel, Arrows to navigate): ", |editor, key, query| {
            let mut moved = false;
            match key {
                Key::Right | Key::Down => {
                    editor.move_cursor(Key::Right);
                    moved = true;
                },
                Key::Left | Key::Up => direction = SearchDirection::Backward,
                _ => direction = SearchDirection::Forward,
            }
            if let Some(position) = editor.document.search(&query, &editor.cursor_position, direction) {
                editor.cursor_position = position;
                editor.scroll();
            } else if moved {
                editor.move_cursor(Key::Left);
            }
        },).unwrap_or(None);
        if query.is_none() {
            self.cursor_position = old_position;
            self.scroll();
        }
    }
}

fn close(e: &std::io::Error) {
    print!("{}", termion::clear::All);
    panic!("{}", e)
}