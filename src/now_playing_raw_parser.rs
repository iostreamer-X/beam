use std::collections::HashMap;

#[derive(Debug)]
pub enum ParsingError {
    UnmatchedBraces,
    MalformedKeyValue(String),
    MalformedValue(String),
}

fn parse_braces(input: &str) -> Result<&str, ParsingError> {
    let input_len = input.len();
    let parsed_output = input.trim_start_matches("{").trim_end_matches("}");
    let are_braces_matched = parsed_output.len() + 2 == input_len;
    if !are_braces_matched {
        return Err(ParsingError::UnmatchedBraces);
    }
    Ok(parsed_output)
}

fn parse_lines(input: &str) -> Result<Vec<&str>, ParsingError> {
    let result: Vec<&str> = input
        .split(";")
        .map(|v| v.trim_start_matches("\n").trim_end_matches("\n"))
        .filter(|v| v.len() > 0)
        .collect();
    Ok(result)
}

fn parse_key(input: &str) -> Result<(&str, &str), ParsingError> {
    let key_terminator_index = input
        .find("=")
        .ok_or(ParsingError::MalformedKeyValue(input.to_string()))?;
    let key = &input[..key_terminator_index];
    let remainder = &input[key_terminator_index + 1..];

    Ok((key.trim(), remainder))
}

fn parse_value(input: &str) -> Result<&str, ParsingError> {
    let trimmed_value = input.trim().trim_start_matches("\"").trim_end_matches("\"");
    Ok(trimmed_value)
}

fn parse_key_value(input: &str) -> Result<(&str, &str), ParsingError> {
    parse_key(input).and_then(|(key, remainder)| parse_value(remainder).map(|value| (key, value)))
}

pub fn parse_raw(input: &String) -> Result<HashMap<&str, &str>, ParsingError> {
    let mut hash_map = HashMap::new();
    parse_braces(input.as_str())
        .and_then(parse_lines)
        .and_then(|lines| {
            lines
                .iter()
                .map(|line| parse_key_value(line))
                .collect::<Result<Vec<(&str, &str)>, ParsingError>>()
        })?
        .iter()
        .for_each(|(key, value)| {
            hash_map.insert(*key, *value);
        });

    Ok(hash_map)
}

#[cfg(test)]
mod tests {
    use crate::now_playing_raw_parser::{parse_braces, parse_key, parse_lines, parse_value};

    use super::parse_raw;

    #[test]
    fn braces_check() {
        let result = parse_braces("{test=\"1\"}").unwrap();
        assert_eq!(result, "test=\"1\"");
    }

    #[test]
    fn lines_check() {
        let result = parse_lines("a=2;b=3").unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "a=2");
        assert_eq!(result[1], "b=3");
    }

    #[test]
    fn key_check() {
        let (key, remainder) = parse_key("a=2").unwrap();
        assert_eq!(key, "a");
        assert_eq!(remainder, "2");
    }

    #[test]
    fn value_check() {
        let value = parse_value("\"2\"").unwrap();
        assert_eq!(value, "2".to_string());
    }

    #[test]
    fn end_to_end_check() {
        let input = "{key1 = \"value1\";\nkey2=value2}".to_string();
        let hash_map = parse_raw(&input).unwrap();
        assert_eq!(hash_map.get("key1").unwrap(), &"value1");
        assert_eq!(hash_map.get("key2").unwrap(), &"value2");
    }
}
