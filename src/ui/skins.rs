use crate::config::SnakeSkin;

pub fn snake_glyph(skin: SnakeSkin, index: usize, total: usize) -> &'static str {
    match skin {
        SnakeSkin::Blocky => {
            if index == 0 {
                "@@"
            } else if index + 1 == total {
                "oo"
            } else {
                "##"
            }
        }
        SnakeSkin::Rhombus => {
            if index == 0 {
                "OO"
            } else if index + 1 == total {
                ">>"
            } else if index.is_multiple_of(2) {
                "<>"
            } else {
                "[]"
            }
        }
    }
}
