use crate::config::SnakeSkin;

pub fn animated_snake_preview(frame_tick: u64, skin: SnakeSkin) -> String {
    match skin {
        SnakeSkin::Blocky => {
            let offset = (frame_tick as usize / 8) % 12;
            let mut row = String::from("            ");
            row.replace_range(offset..offset + 1, "@");
            for tail in 1..5 {
                if offset >= tail {
                    row.replace_range(offset - tail..offset - tail + 1, "o");
                }
            }
            row.push_str("   *");
            row
        }
        SnakeSkin::Rhombus => "O ◆◈◆◈◆◈◆◈◆◈ ≫   *".to_string(),
    }
}

pub fn snake_glyph(skin: SnakeSkin, index: usize, total: usize) -> &'static str {
    match skin {
        SnakeSkin::Blocky => {
            if index == 0 {
                "@"
            } else if index + 1 == total {
                "o"
            } else {
                "#"
            }
        }
        SnakeSkin::Rhombus => {
            if index == 0 {
                "O"
            } else if index + 1 == total {
                "≫"
            } else if index.is_multiple_of(2) {
                "◈"
            } else {
                "◆"
            }
        }
    }
}
