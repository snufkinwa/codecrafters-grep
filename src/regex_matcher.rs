use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone)]
pub enum Pattern {
    Literal(String),
    Digit,
    Alphanumeric,
    AnyChar,
    Start,
    End,
    CharGroup(Vec<char>, bool), 
}

// Display implementation for BasicPattern
impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pattern::Literal(s) => write!(f, "{}", s),
            Pattern::Digit => write!(f, "\\d"),
            Pattern::Alphanumeric => write!(f, "\\w"),
            Pattern::AnyChar => write!(f, "."),
            Pattern::Start => write!(f, "^"),
            Pattern::End => write!(f, "$"),
            Pattern::CharGroup(chars, is_negative) => {
                let mut s = String::new();
                s.push('[');
                if *is_negative {
                    s.push('^');
                }
                s.extend(chars.iter());
                s.push(']');
                write!(f, "{}", s)
            }
        }
    }
}

pub fn parse_pattern(pattern: &str) -> Vec<Pattern> {
    let mut patterns = Vec::new();
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(escaped) = chars.next() {
                    match escaped {
                        'd' => patterns.push(Pattern::Digit),
                        'w' => patterns.push(Pattern::Alphanumeric),
                        '\\' => patterns.push(Pattern::Literal(String::from("\\"))),
                        _ => patterns.push(Pattern::Literal(escaped.to_string())),
                    }
                }
            }
            '.' => patterns.push(Pattern::AnyChar),
            '^' => patterns.push(Pattern::Start),
            '$' => patterns.push(Pattern::End),
            '[' => {
                let is_negative = chars.peek() == Some(&'^');
                if is_negative {
                    chars.next(); 
                }
                let mut group = Vec::new();
                while let Some(group_char) = chars.next() {
                    if group_char == ']' {
                        break;
                    }
                    group.push(group_char);
                }
                patterns.push(Pattern::CharGroup(group, is_negative));
            }
            _ => {
                let mut literal = String::new();
                literal.push(c);
                while let Some(&next_c) = chars.peek() {
                    if next_c != '\\' && next_c != '.' && next_c != '^' && next_c != '$' && next_c != '[' {
                        literal.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                patterns.push(Pattern::Literal(literal));
            }
        }
    }
    patterns
}

fn match_class(pattern: &Pattern, input_chars: &mut Peekable<Chars>) -> bool {
    match pattern {
        Pattern::Digit => {
            input_chars.next().map_or(false, |c| c.is_digit(10))
        }
        Pattern::Alphanumeric => {
            input_chars.next().map_or(false, |c| c.is_alphanumeric())
        }
        Pattern::AnyChar => input_chars.next().is_some(),
        Pattern::CharGroup(group, is_negative) => {
            input_chars.next().map_or(false, |c| group.contains(&c) != *is_negative)
        }
        Pattern::Start | Pattern::End => true, 
        _ => false,
    }
}

fn match_literal(literal: &str, input_chars: &mut Peekable<Chars>) -> bool {
    let mut literal_chars = literal.chars();
    while let Some(lit_char) = literal_chars.next() {
        loop {
            match input_chars.next() {
                Some(input_char) if input_char == lit_char => break,
                Some(' ') => continue,
                _ => return false,
            }
        }
    }
    true
}

fn match_from_current_position(input_chars: &mut Peekable<Chars>, patterns: &[Pattern], is_start: bool) -> bool {
    let mut input_clone = input_chars.clone();
    for (i, pattern) in patterns.iter().enumerate() {
        match pattern {
            Pattern::Literal(ref literal) => {
                if !match_literal(literal, &mut input_clone) {
                    return false;
                }
            }
            Pattern::Start => {
                if i != 0 || !is_start {
                    return false;
                }
            }
            Pattern::End => {
                if i != patterns.len() - 1 || input_clone.peek().is_some() {
                    return false;
                }
            }
            _ => {
                if !match_class(pattern, &mut input_clone) {
                    return false;
                }
            }
        }
    }
    *input_chars = input_clone;
    true
}

pub fn match_pattern(input_line: &str, pattern_str: &str) -> bool {
    let patterns = parse_pattern(pattern_str);
    let mut input_chars = input_line.chars().peekable();

    let starts_with_anchor = matches!(patterns.first(), Some(Pattern::Start));
    let ends_with_anchor = matches!(patterns.last(), Some(Pattern::End));

    if starts_with_anchor && ends_with_anchor {
        return match_from_current_position(&mut input_chars, &patterns, true);
    } else if starts_with_anchor {
        return match_from_current_position(&mut input_chars, &patterns, true);
    } else if ends_with_anchor {
        while input_chars.peek().is_some() {
            let mut clone = input_chars.clone();
            if match_from_current_position(&mut clone, &patterns, false) && clone.peek().is_none() {
                return true;
            }
            input_chars.next();
        }
    } else {
        while input_chars.peek().is_some() {
            if match_from_current_position(&mut input_chars.clone(), &patterns, false) {
                return true;
            }
            input_chars.next();
        }
    }

    false
}
