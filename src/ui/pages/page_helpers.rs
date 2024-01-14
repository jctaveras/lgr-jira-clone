use ellipse::Ellipse;

pub fn get_column_string(text: &str, width: usize) -> String {
    match text.len().cmp(&width) {
        std::cmp::Ordering::Equal => text.to_string(),
        std::cmp::Ordering::Greater => {
            return match width {
                0 => "".to_string(),
                1 => ".".to_string(),
                2 => "..".to_string(),
                3 => "...".to_string(),
                _ => text.truncate_ellipse(width - 3).to_string(),
            };
        }
        std::cmp::Ordering::Less => {
            let mut column_string = text.to_owned();

            (0..(width - text.len())).for_each(|_| column_string.push(' '));
            return column_string;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_column_string() {
        let text1 = "";
        let text2 = "test";
        let text3 = "testme";
        let text4 = "testmetest";

        let width = 0;

        assert_eq!(get_column_string(text4, width), "".to_owned());

        let width = 1;

        assert_eq!(get_column_string(text4, width), ".".to_owned());

        let width = 2;

        assert_eq!(get_column_string(text4, width), "..".to_owned());

        let width = 3;

        assert_eq!(get_column_string(text4, width), "...".to_owned());

        let width = 4;

        assert_eq!(get_column_string(text4, width), "t...".to_owned());

        let width = 6;

        assert_eq!(get_column_string(text1, width), "      ".to_owned());
        assert_eq!(get_column_string(text2, width), "test  ".to_owned());
        assert_eq!(get_column_string(text3, width), "testme".to_owned());
        assert_eq!(get_column_string(text4, width), "tes...".to_owned());
    }
}
