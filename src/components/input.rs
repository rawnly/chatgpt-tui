use crate::cursor::Cursor;

#[derive(Debug, Clone)]
pub struct Input {
    cursor: Cursor,
    pub max_length: usize,
    pub text: String,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            cursor: Cursor::default(),
            max_length: 250,
            text: String::new(),
        }
    }
}

impl Input {
    pub fn new(max_length: usize) -> Self {
        Self {
            max_length,
            cursor: Cursor::default(),
            text: String::new(),
        }
    }

    pub fn set_value(&mut self, text: String) {
        if text.len() > self.max_length {
            self.text = text.split_at(self.max_length).0.to_string();
        } else {
            self.text = text
        }

        self.cursor.update_input_length(&self.text);
        self.cursor.move_to_end();
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor.reset();
    }

    pub fn right(&mut self) {
        self.cursor.right();
    }

    pub fn left(&mut self) {
        self.cursor.left();
    }

    pub fn cursor_position(&self) -> usize {
        self.cursor.position
    }

    pub fn delete(&mut self) {
        let is_not_cursor_leftmost = !self.cursor.is_at_start();

        if is_not_cursor_leftmost {
            let current_index = self.cursor.position;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.text.chars().take(from_left_to_current_index);

            let after_char_to_delete = self.text.chars().skip(current_index);

            self.text = before_char_to_delete.chain(after_char_to_delete).collect();
            self.cursor.update_input_length(&self.text);
            self.cursor.left();
        }
    }

    pub fn insert(&mut self, c: char) {
        if self.text.len() >= self.max_length {
            return;
        }

        self.text.insert(self.cursor.position, c);
        self.cursor.update_input_length(&self.text);
        self.cursor.right();
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}
