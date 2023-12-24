use ratatui::widgets::*;

#[derive(Debug, Clone)]
pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> Default for StatefulList<T> {
    fn default() -> Self {
        Self {
            state: ListState::default(),
            items: Vec::new(),
        }
    }
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        if !self.items.is_empty() {
            self.state.select(Some(i))
        }
    }

    pub fn select(&mut self, i: usize) {
        if !self.items.is_empty() && self.items.len() > i {
            self.state.select(Some(i))
        }
    }

    pub fn select_last(&mut self) {
        if self.items.is_empty() {
            self.unselect();
            return;
        }

        let i = self.items.len() - 1;
        self.state.select(Some(i));
    }

    pub fn prev(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };

        if !self.items.is_empty() {
            self.state.select(Some(i))
        }
    }

    pub fn unselect(&mut self) {
        self.state.select(None)
    }

    pub fn select_first(&mut self) {
        if !self.items.is_empty() {
            self.state.select(Some(0))
        }
    }
}
