use std::time::{SystemTime, UNIX_EPOCH};
pub fn day() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() / 86400)
}

pub fn palette(text: &str) -> Option<[u32; 3]> {
    let colors = text
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    if colors.len() != 3 {
        return None;
    }
    let mut result = [0; 3];
    for (out, color) in result.iter_mut().zip(colors) {
        let value = color.trim_start_matches('#');
        if value.len() != 6 {
            return None;
        }
        *out = u32::from_str_radix(value, 16).ok()?;
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn palette_input_accepts_user_separators_and_rejects_partial_or_invalid_colors() {
        assert_eq!(
            palette(" #B0DAF5, #f8cddc  DBD1F6 "),
            Some([0xb0daf5, 0xf8cddc, 0xdbd1f6])
        );
        for text in [
            "#fff #000 #abc",
            "#123456 #654321",
            "#123456 #654321 #gggggg",
            "#123456 #654321 #abcdef #000000",
        ] {
            assert_eq!(palette(text), None);
        }
    }
}
