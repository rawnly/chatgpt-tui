use crate::cursor::Cursor;

#[derive(Debug, Default, Clone)]
pub struct Input {
    cursor: Cursor,
    pub text: String,
}

impl Input {
    pub fn set_value(&mut self, text: String) {
        self.text = text;
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
        self.text.insert(self.cursor.position, c);
        self.cursor.update_input_length(&self.text);
        self.cursor.right();
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}
