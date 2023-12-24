#[derive(Debug, Default, Clone)]
pub struct Cursor {
    pub position: usize,
    pub input_length: usize,
}

impl Cursor {
    pub fn clamp(&self, value: usize) -> usize {
        value.clamp(0, self.input_length)
    }

    pub fn reset(&mut self) {
        self.move_to_start();
        self.input_length = 0;
    }

    pub fn left(&mut self) {
        self.position = self.clamp(self.position.saturating_sub(1));
    }

    pub fn right(&mut self) {
        self.position = self.clamp(self.position.saturating_add(1));
    }

    pub fn update_input_length(&mut self, input: &str) {
        self.input_length = input.chars().count();
    }

    pub fn is_at_start(&self) -> bool {
        self.position == 0
    }

    pub fn move_to_end(&mut self) {
        self.position = self.input_length;
    }

    pub fn move_to_start(&mut self) {
        self.position = 0;
    }
}
