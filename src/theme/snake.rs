use ratatui::style::Style;

use super::model::Theme;

impl Theme {
    /// Select the style for a snake segment based on its position.
    pub fn snake_segment(&self, index: usize, total: usize) -> Style {
        if index == 0 {
            return self.snake_head;
        }
        if index + 1 == total {
            return self.snake_tail;
        }
        if total <= 3 {
            return self.snake_body;
        }

        let t = index as f32 / (total - 1).max(1) as f32;
        if t < 0.35 {
            self.snake_body
        } else {
            self.snake_tail
        }
    }
}
