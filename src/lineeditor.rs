use super::commandlist::*;
use unicode_width::*;

#[derive(Debug, Clone)]
pub struct EditorState {
    lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
}
pub enum EditorEvent {
    NewCharacter(char),
    NewLine,
    Backspace,
    Delete,
    Clear,
    GoLeft,
    GoRight,
    GoUp,
    GoDown,
    Home,
    End,
    KillWordBack,
}

impl EditorState {
    pub fn new() -> EditorState {
        EditorState {
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
        }
    }

    pub fn content_to_commandentry(&self) -> CommandEntry {
        CommandEntry::new(&self.lines)
    }

    pub fn load_commandentry(&mut self, entry: &CommandEntry) {
        self.set_content(&entry.lines());
    }

    pub fn set_content(&mut self, new_content: &Vec<String>) {
        // prevent setting _no_ lines, which would crash
        self.lines = if new_content.is_empty() {
            vec![String::new()]
        } else {
            new_content.clone()
        };
        self.cursor_line = self.lines.len() - 1;
        self.cursor_col = self.current_line().len();
    }

    pub fn content_str(&self) -> String {
        self.lines.join("").to_owned()
    }

    pub fn content_lines(&self) -> Vec<String> {
        self.lines.to_owned()
    }

    pub fn current_line(&self) -> &str {
        &self.lines[self.cursor_line]
    }

    pub fn displayed_cursor_column(&self) -> usize {
        UnicodeWidthStr::width(&self.current_line()[..self.cursor_col])
    }

    fn current_line_mut(&mut self) -> &mut String {
        &mut self.lines[self.cursor_line]
    }

    fn next_char_index(&self) -> usize {
        if self.cursor_col == self.current_line().len() {
            return self.cursor_col;
        }
        let mut new_cursor = self.cursor_col + 1;
        while let None = self.current_line().get(new_cursor..) {
            new_cursor += 1;
        }
        new_cursor
    }

    fn prev_char_index(&self) -> usize {
        if self.cursor_col == 0 {
            return 0;
        }
        let mut new_cursor = self.cursor_col - 1;
        while let None = self.current_line().get(new_cursor..) {
            new_cursor -= 1;
        }
        new_cursor
    }

    /// go to another line, keeping the cursor column the same if possible,
    /// otherwise going to the last column of the line
    fn goto_line(&mut self, line_nr: usize) {
        assert!(line_nr < self.lines.len());
        self.cursor_line = line_nr;
        if self.cursor_col >= self.current_line().len() {
            self.cursor_col = self.current_line().len()
        }
    }

    pub fn apply_event(&mut self, event: EditorEvent) {
        match event {
            EditorEvent::NewCharacter(c) => {
                let cursor_col = self.cursor_col;
                self.current_line_mut().insert(cursor_col, c);
                self.cursor_col = self.next_char_index();
            }
            EditorEvent::NewLine => {
                let cursor_col = self.cursor_col;
                let rest_of_line = self.current_line_mut().split_off(cursor_col);
                self.lines.insert(self.cursor_line + 1, rest_of_line);
                self.goto_line(self.cursor_line + 1);
                self.cursor_col = 0;
            }
            EditorEvent::Backspace => {
                if self.cursor_col > 0 {
                    // delete character
                    let new_cursor = self.prev_char_index();
                    self.current_line_mut().remove(new_cursor);
                    self.cursor_col = new_cursor;
                } else if self.cursor_line > 0 {
                    // backspace at start of line: delete newline
                    let removed_line = self.lines.remove(self.cursor_line);
                    self.cursor_line -= 1;
                    self.cursor_col = self.current_line().len();
                    self.current_line_mut().push_str(&removed_line);
                }
            }
            EditorEvent::Delete => {
                if self.cursor_col < self.current_line().len() {
                    // delete character
                    let cursor_col = self.cursor_col;
                    self.current_line_mut().remove(cursor_col);
                } else if self.cursor_line < self.lines.len() - 1 {
                    // delete at end of line: delete newline
                    let removed_line = self.lines.remove(self.cursor_line + 1);
                    self.current_line_mut().push_str(&removed_line);
                }
            }

            EditorEvent::Clear => {
                self.set_content(&vec![String::new()]);
            }

            EditorEvent::GoLeft => {
                if self.cursor_col > 0 {
                    self.cursor_col = self.prev_char_index();
                } else if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.cursor_col = self.current_line().len();
                }
            }
            EditorEvent::GoRight => {
                if self.cursor_col < self.current_line().len() {
                    self.cursor_col = self.next_char_index();
                } else if self.cursor_line < self.lines.len() - 1 {
                    self.cursor_line += 1;
                    self.cursor_col = 0;
                }
            }
            EditorEvent::GoUp if self.cursor_line > 0 => self.goto_line(self.cursor_line - 1),
            EditorEvent::GoDown if self.cursor_line < self.lines.len() - 1 => self.goto_line(self.cursor_line + 1),
            EditorEvent::Home => self.cursor_col = 0,
            EditorEvent::End => self.cursor_col = self.current_line().len(),

            EditorEvent::KillWordBack if !self.current_line().is_empty() => {
                while let Some(c) = self.current_line().to_owned().get(self.prev_char_index()..self.cursor_col) {
                    let cursor_col = self.prev_char_index();
                    self.cursor_col = cursor_col;
                    self.current_line_mut().remove(cursor_col);
                    if c == " " || c == "/" || c == "\\" || c == ":" || c == "_" || c == "-" || self.cursor_col == 0 {
                        break;
                    }
                }
            }
            _ => {}
        }
    }
}

pub mod test {
    #[allow(unused_imports)]
    use super::*;
    #[test]
    pub fn test_lineeditor_ascii() {
        let mut le = EditorState::new();
        assert_eq!(le.content_str(), "");

        le.apply_event(EditorEvent::NewCharacter('a'));
        assert_eq!(le.content_str(), "a");

        le.apply_event(EditorEvent::NewCharacter('a'));
        assert_eq!(le.content_str(), "aa");
        assert_eq!(le.displayed_cursor_column(), 2);

        le.apply_event(EditorEvent::Backspace);
        assert_eq!(le.content_str(), "a");
        assert_eq!(le.displayed_cursor_column(), 1);

        le.apply_event(EditorEvent::Backspace);
        assert_eq!(le.content_str(), "");
        assert_eq!(le.displayed_cursor_column(), 0);

        le.apply_event(EditorEvent::Backspace);
        assert_eq!(le.content_str(), "");
        assert_eq!(le.displayed_cursor_column(), 0);

        le.apply_event(EditorEvent::NewCharacter('a'));
        assert_eq!(le.content_str(), "a");
        assert_eq!(le.displayed_cursor_column(), 1);

        le.apply_event(EditorEvent::GoLeft);
        assert_eq!(le.displayed_cursor_column(), 0);

        le.apply_event(EditorEvent::Delete);
        assert_eq!(le.content_str(), "");
        assert_eq!(le.displayed_cursor_column(), 0);

        le.apply_event(EditorEvent::Delete);
        assert_eq!(le.content_str(), "");
        assert_eq!(le.displayed_cursor_column(), 0);
    }

    #[test]
    pub fn test_advanced() {
        let mut le = EditorState::new();
        le.set_content(&vec!["as".to_string()]);
        assert_eq!(le.content_str(), "as");
        assert_eq!(le.displayed_cursor_column(), 2 as usize);

        le.apply_event(EditorEvent::KillWordBack);
        assert_eq!(le.content_str(), "");
        assert_eq!(le.displayed_cursor_column(), 0 as usize);

        le.set_content(&vec!["as as as".to_string()]);
        assert_eq!(le.content_str(), "as as as");
        assert_eq!(le.displayed_cursor_column(), 8 as usize);

        le.apply_event(EditorEvent::KillWordBack);
        assert_eq!(le.content_str(), "as as");
        assert_eq!(le.displayed_cursor_column(), 5 as usize);
    }

    #[test]
    pub fn test_lineeditor_umlaut() {
        let mut le = EditorState::new();
        assert_eq!(le.content_str(), "");

        le.apply_event(EditorEvent::NewCharacter('ä'));
        assert_eq!(le.content_str(), "ä");
        assert_eq!(le.displayed_cursor_column(), 1);
        le.apply_event(EditorEvent::NewCharacter('ä'));
        assert_eq!(le.content_str(), "ää");
        assert_eq!(le.displayed_cursor_column(), 2);

        le.apply_event(EditorEvent::GoLeft);
        assert_eq!(le.displayed_cursor_column(), 1);
    }

    #[test]
    pub fn test_multiline() {
        let mut le = EditorState::new();

        le.apply_event(EditorEvent::NewLine);
        assert_eq!(le.content_lines(), vec!["", ""]);
        assert_eq!(le.cursor_line, 1);

        le.apply_event(EditorEvent::NewCharacter('a'));
        assert_eq!(le.content_lines(), vec!["", "a"]);
        assert_eq!(le.cursor_line, 1);

        le.apply_event(EditorEvent::GoUp);
        assert_eq!(le.content_lines(), vec!["", "a"]);
        assert_eq!(le.cursor_line, 0);

        le.apply_event(EditorEvent::GoDown);
        assert_eq!(le.content_lines(), vec!["", "a"]);
        assert_eq!(le.cursor_line, 1);

        le.set_content(&vec!["a".into(), "b".into()]);
        assert_eq!(le.content_lines(), vec!["a", "b"]);
        assert_eq!(le.cursor_line, 1);
        le.apply_event(EditorEvent::Home);
        le.apply_event(EditorEvent::Backspace);
        assert_eq!(le.cursor_line, 0);
        assert_eq!(le.content_lines(), vec!["ab"]);

        le.set_content(&vec!["abc".into(), "a".into()]);
        assert_eq!(le.cursor_line, 1);
        assert_eq!(le.cursor_col, 1);

        le.apply_event(EditorEvent::GoUp);
        assert_eq!(le.cursor_col, 1);
        assert_eq!(le.cursor_line, 0);
        le.apply_event(EditorEvent::End);
        assert_eq!(le.cursor_col, 3);
        le.apply_event(EditorEvent::GoDown);
        assert_eq!(le.cursor_line, 1);
        assert_eq!(le.cursor_col, 1);
    }
}
