use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::theme::Theme;

pub fn draw_oxide_rain(frame: &mut Frame, area: Rect, theme: &Theme, frame_tick: u64) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    const GLYPHS: &[u8] = b"RUST01<>[]{}fnletmut";
    let columns = area.width;

    for x in 0..columns {
        let speed = 1 + (x as u64 % 4);
        let head = ((frame_tick / speed + x as u64 * 7) % area.height as u64) as u16;
        let stream_len = 4 + (x % 7);

        for tail in 0..stream_len {
            let y = head.saturating_sub(tail);
            if y >= area.height {
                continue;
            }

            let glyph_index =
                ((x as u64 * 13 + y as u64 * 5 + frame_tick) % GLYPHS.len() as u64) as usize;
            let ch = GLYPHS[glyph_index] as char;
            let style = if tail == 0 {
                theme.rain_head
            } else if tail < 3 {
                theme.rain_mid
            } else {
                theme.rain_tail
            };
            let rect = Rect {
                x: area.x + x,
                y: area.y + y,
                width: 1,
                height: 1,
            };
            frame.render_widget(Paragraph::new(ch.to_string()).style(style), rect);
        }
    }
}
