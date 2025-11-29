//! JSON file analysis

use super::AnalysisError;
use crate::types::{JsonAnalysis, JsonRootType};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Analyze a JSON file
///
/// Validates JSON syntax and extracts structural information.
/// Uses a simple recursive descent parser built from scratch.
///
/// # Arguments
///
/// * `path` - Path to the JSON file
///
/// # Returns
///
/// A JsonAnalysis struct with validation results and structure info.
pub fn analyze_json(path: &Path) -> Result<JsonAnalysis, AnalysisError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read entire file into string
    // For very large JSON files, this could be optimized with streaming
    let mut content = String::new();
    for line_result in reader.lines() {
        let line = line_result?;
        content.push_str(&line);
        content.push('\n');
    }

    // Parse JSON
    let mut parser = JsonParser::new(&content);
    match parser.parse() {
        Ok(value) => {
            let (root_type, top_level_keys) = match value {
                JsonValue::Object(keys) => (JsonRootType::Object, keys),
                JsonValue::Array(_) => (JsonRootType::Array, Vec::new()),
                _ => return Ok(JsonAnalysis::invalid()),
            };

            Ok(JsonAnalysis {
                is_valid: true,
                root_type,
                top_level_keys,
            })
        }
        Err(_) => Ok(JsonAnalysis::invalid()),
    }
}

/// JSON value representation for parsing
#[derive(Debug)]
#[allow(dead_code)]
enum JsonValue {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<String>), // Store only keys for top-level objects
}

/// Simple JSON parser
struct JsonParser {
    chars: Vec<char>,
    pos: usize,
}

impl JsonParser {
    fn new(input: &str) -> Self {
        JsonParser {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    fn parse(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        self.parse_value()
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();

        if self.pos >= self.chars.len() {
            return Err("Unexpected end of input".to_string());
        }

        match self.chars[self.pos] {
            'n' => self.parse_null(),
            't' | 'f' => self.parse_boolean(),
            '"' => self.parse_string(),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            '-' | '0'..='9' => self.parse_number(),
            _ => Err(format!("Unexpected character: {}", self.chars[self.pos])),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.consume_literal("null") {
            Ok(JsonValue::Null)
        } else {
            Err("Invalid null".to_string())
        }
    }

    fn parse_boolean(&mut self) -> Result<JsonValue, String> {
        if self.consume_literal("true") {
            Ok(JsonValue::Boolean(true))
        } else if self.consume_literal("false") {
            Ok(JsonValue::Boolean(false))
        } else {
            Err("Invalid boolean".to_string())
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;

        // Optional minus
        if self.pos < self.chars.len() && self.chars[self.pos] == '-' {
            self.pos += 1;
        }

        // Digits
        if self.pos >= self.chars.len() || !self.chars[self.pos].is_ascii_digit() {
            return Err("Invalid number".to_string());
        }

        while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
            self.pos += 1;
        }

        // Optional decimal part
        if self.pos < self.chars.len() && self.chars[self.pos] == '.' {
            self.pos += 1;
            while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }

        // Optional exponent
        if self.pos < self.chars.len() && (self.chars[self.pos] == 'e' || self.chars[self.pos] == 'E') {
            self.pos += 1;
            if self.pos < self.chars.len() && (self.chars[self.pos] == '+' || self.chars[self.pos] == '-') {
                self.pos += 1;
            }
            while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }

        let num_str: String = self.chars[start..self.pos].iter().collect();
        num_str
            .parse::<f64>()
            .map(JsonValue::Number)
            .map_err(|_| "Invalid number format".to_string())
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        if self.pos >= self.chars.len() || self.chars[self.pos] != '"' {
            return Err("Expected string".to_string());
        }
        self.pos += 1; // Skip opening quote

        let mut result = String::new();

        while self.pos < self.chars.len() {
            match self.chars[self.pos] {
                '"' => {
                    self.pos += 1;
                    return Ok(JsonValue::String(result));
                }
                '\\' => {
                    self.pos += 1;
                    if self.pos >= self.chars.len() {
                        return Err("Unterminated string".to_string());
                    }
                    match self.chars[self.pos] {
                        '"' | '\\' | '/' => result.push(self.chars[self.pos]),
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        _ => result.push(self.chars[self.pos]),
                    }
                    self.pos += 1;
                }
                c => {
                    result.push(c);
                    self.pos += 1;
                }
            }
        }

        Err("Unterminated string".to_string())
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        if self.pos >= self.chars.len() || self.chars[self.pos] != '[' {
            return Err("Expected array".to_string());
        }
        self.pos += 1;

        let mut elements = Vec::new();
        self.skip_whitespace();

        if self.pos < self.chars.len() && self.chars[self.pos] == ']' {
            self.pos += 1;
            return Ok(JsonValue::Array(elements));
        }

        loop {
            elements.push(self.parse_value()?);
            self.skip_whitespace();

            if self.pos >= self.chars.len() {
                return Err("Unterminated array".to_string());
            }

            match self.chars[self.pos] {
                ',' => {
                    self.pos += 1;
                    self.skip_whitespace();
                }
                ']' => {
                    self.pos += 1;
                    return Ok(JsonValue::Array(elements));
                }
                _ => return Err("Expected comma or closing bracket".to_string()),
            }
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        if self.pos >= self.chars.len() || self.chars[self.pos] != '{' {
            return Err("Expected object".to_string());
        }
        self.pos += 1;

        let mut keys = Vec::new();
        self.skip_whitespace();

        if self.pos < self.chars.len() && self.chars[self.pos] == '}' {
            self.pos += 1;
            return Ok(JsonValue::Object(keys));
        }

        loop {
            self.skip_whitespace();

            // Parse key (must be string)
            let key = self.parse_string()?;
            if let JsonValue::String(k) = key {
                keys.push(k);
            }

            self.skip_whitespace();

            if self.pos >= self.chars.len() || self.chars[self.pos] != ':' {
                return Err("Expected colon".to_string());
            }
            self.pos += 1;

            // Parse value (but don't store it for efficiency)
            self.parse_value()?;

            self.skip_whitespace();

            if self.pos >= self.chars.len() {
                return Err("Unterminated object".to_string());
            }

            match self.chars[self.pos] {
                ',' => {
                    self.pos += 1;
                    self.skip_whitespace();
                }
                '}' => {
                    self.pos += 1;
                    return Ok(JsonValue::Object(keys));
                }
                _ => return Err("Expected comma or closing brace".to_string()),
            }
        }
    }

    fn consume_literal(&mut self, literal: &str) -> bool {
        let chars: Vec<char> = literal.chars().collect();
        if self.pos + chars.len() > self.chars.len() {
            return false;
        }

        for (i, &c) in chars.iter().enumerate() {
            if self.chars[self.pos + i] != c {
                return false;
            }
        }

        self.pos += chars.len();
        true
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() {
        let mut parser = JsonParser::new("null");
        let result = parser.parse().unwrap();
        matches!(result, JsonValue::Null);
    }

    #[test]
    fn test_parse_boolean() {
        let mut parser = JsonParser::new("true");
        let result = parser.parse().unwrap();
        matches!(result, JsonValue::Boolean(true));
    }

    #[test]
    fn test_parse_number() {
        let mut parser = JsonParser::new("123.456");
        let result = parser.parse().unwrap();
        matches!(result, JsonValue::Number(_));
    }

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new(r#""hello""#);
        let result = parser.parse().unwrap();
        if let JsonValue::String(s) = result {
            assert_eq!(s, "hello");
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn test_parse_array() {
        let mut parser = JsonParser::new("[1, 2, 3]");
        let result = parser.parse().unwrap();
        matches!(result, JsonValue::Array(_));
    }

    #[test]
    fn test_parse_object() {
        let mut parser = JsonParser::new(r#"{"key": "value"}"#);
        let result = parser.parse().unwrap();
        if let JsonValue::Object(keys) = result {
            assert_eq!(keys, vec!["key"]);
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_analyze_valid_json() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_json_valid");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.json");
        std::fs::write(&file_path, r#"{"name": "test", "value": 123}"#).unwrap();

        let result = analyze_json(&file_path).unwrap();
        assert!(result.is_valid);
        assert_eq!(result.root_type, JsonRootType::Object);
        assert_eq!(result.top_level_keys.len(), 2);

        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_analyze_invalid_json() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_json_invalid");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.json");
        std::fs::write(&file_path, r#"{"invalid": }"#).unwrap();

        let result = analyze_json(&file_path).unwrap();
        assert!(!result.is_valid);

        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_analyze_json_array() {
        let temp_dir = std::env::temp_dir().join("genesis_preflight_json_array");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.json");
        std::fs::write(&file_path, r#"[1, 2, 3]"#).unwrap();

        let result = analyze_json(&file_path).unwrap();
        assert!(result.is_valid);
        assert_eq!(result.root_type, JsonRootType::Array);

        std::fs::remove_dir_all(temp_dir).ok();
    }
}
