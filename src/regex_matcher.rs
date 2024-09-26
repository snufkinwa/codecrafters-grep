use std::collections::HashMap;
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
    OneOrMore(Box<Pattern>),    
    ZeroOrOne(Box<Pattern>),   
    Alternation(Vec<Pattern>), 
    Group(Vec<Pattern>),        
    BackReference(usize),       
}

// Display implementation for Pattern
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
            Pattern::OneOrMore(p) => write!(f, "{}+", p),
            Pattern::ZeroOrOne(p) => write!(f, "{}?", p),
            Pattern::Alternation(alternatives) => {
                let mut s = String::new();
                s.push('(');
                let len = alternatives.len();
                for (i, alt) in alternatives.iter().enumerate() {
                    s.push_str(&alt.to_string());
                    if i < len - 1 {
                        s.push('|');
                    }
                }
                s.push(')');
                write!(f, "{}", s)
            }
            Pattern::Group(subpatterns) => {
                let mut s = String::new();
                s.push('(');
                for subpattern in subpatterns {
                    s.push_str(&subpattern.to_string());
                }
                s.push(')');
                write!(f, "{}", s)
            }
            Pattern::BackReference(n) => write!(f, "\\{}", n),
        }
    }
}

pub fn parse_pattern(pattern: &str) -> Vec<Pattern> {
    let mut patterns = Vec::new();
    let mut chars = pattern.chars().peekable();
    let mut literal_buffer = String::new(); 

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                // Peek the next character to determine the type
                if let Some(&next_char) = chars.peek() {
                    if next_char.is_digit(10) {
                        // It's a backreference like \1, \2, etc.
                        chars.next(); // Consume the digit
                        let group_num = next_char.to_digit(10).unwrap() as usize;
                        // Flush the buffer before handling backreference
                        if !literal_buffer.is_empty() {
                            patterns.push(Pattern::Literal(literal_buffer.clone()));
                            literal_buffer.clear();
                        }
                        patterns.push(Pattern::BackReference(group_num));
                        continue;
                    }
                }

                // Handle escaped characters
                if !literal_buffer.is_empty() {
                    patterns.push(Pattern::Literal(literal_buffer.clone()));
                    literal_buffer.clear();
                }
                if let Some(escaped) = chars.next() {
                    let pattern = match escaped {
                        'd' => Pattern::Digit,
                        'w' => Pattern::Alphanumeric,
                        '\\' => Pattern::Literal("\\".to_string()),
                        _ => Pattern::Literal(escaped.to_string()),
                    };
                    patterns.push(pattern);
                }
            }
            '.' => {
                // Flush the buffer before handling special patterns
                if !literal_buffer.is_empty() {
                    patterns.push(Pattern::Literal(literal_buffer.clone()));
                    literal_buffer.clear();
                }
                patterns.push(Pattern::AnyChar);
            }
            '^' => {
                // Flush the buffer before handling special patterns
                if !literal_buffer.is_empty() {
                    patterns.push(Pattern::Literal(literal_buffer.clone()));
                    literal_buffer.clear();
                }
                patterns.push(Pattern::Start);
            }
            '$' => {
                // Flush the buffer before handling special patterns
                if !literal_buffer.is_empty() {
                    patterns.push(Pattern::Literal(literal_buffer.clone()));
                    literal_buffer.clear();
                }
                patterns.push(Pattern::End);
            }
            '[' => {
                // Flush the buffer before handling special patterns
                if !literal_buffer.is_empty() {
                    patterns.push(Pattern::Literal(literal_buffer.clone()));
                    literal_buffer.clear();
                }
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
            '(' => {
                // Flush the buffer before handling groups
                if !literal_buffer.is_empty() {
                    patterns.push(Pattern::Literal(literal_buffer.clone()));
                    literal_buffer.clear();
                }
                // Parse the group
                let mut group_pattern = String::new();
                let mut depth = 1;
                while let Some(next_char) = chars.next() {
                    if next_char == '(' {
                        depth += 1;
                        group_pattern.push(next_char);
                    } else if next_char == ')' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        group_pattern.push(next_char);
                    } else {
                        group_pattern.push(next_char);
                    }
                }
                let group_patterns = parse_group_pattern(&group_pattern);
                patterns.push(Pattern::Group(group_patterns));
            }
            '|' => {
                if !literal_buffer.is_empty() {
                    patterns.push(Pattern::Literal(literal_buffer.clone()));
                    literal_buffer.clear();
                }
                patterns.push(Pattern::Literal("|".to_string()));
            }
            '+' => {
                if !literal_buffer.is_empty() {
                    if literal_buffer.len() > 1 {
                        let mut chars_buffer: Vec<char> = literal_buffer.chars().collect();
                        let last_char = chars_buffer.pop().unwrap();
                        if !chars_buffer.is_empty() {
                            let remaining = chars_buffer.into_iter().collect::<String>();
                            patterns.push(Pattern::Literal(remaining));
                        }
                        let literal = Pattern::Literal(last_char.to_string());
                        let one_or_more = Pattern::OneOrMore(Box::new(literal));
                        patterns.push(one_or_more);
                    } else {
                        let literal = Pattern::Literal(literal_buffer.clone());
                        let one_or_more = Pattern::OneOrMore(Box::new(literal));
                        patterns.push(one_or_more);
                    }
                    literal_buffer.clear();
                } else if let Some(last) = patterns.pop() {
                    let one_or_more = Pattern::OneOrMore(Box::new(last));
                    patterns.push(one_or_more);
                } else {
                    patterns.push(Pattern::Literal("+".to_string()));
                }
            }
            '?' => {
                if !literal_buffer.is_empty() {
                    if literal_buffer.len() > 1 {
                        let mut chars_buffer: Vec<char> = literal_buffer.chars().collect();
                        let last_char = chars_buffer.pop().unwrap();
                        if !chars_buffer.is_empty() {
                            let remaining = chars_buffer.into_iter().collect::<String>();
                            patterns.push(Pattern::Literal(remaining));
                        }
                        let literal = Pattern::Literal(last_char.to_string());
                        let zero_or_one = Pattern::ZeroOrOne(Box::new(literal));
                        patterns.push(zero_or_one);
                    } else {
                        let literal = Pattern::Literal(literal_buffer.clone());
                        let zero_or_one = Pattern::ZeroOrOne(Box::new(literal));
                        patterns.push(zero_or_one);
                    }
                    literal_buffer.clear();
                } else if let Some(last) = patterns.pop() {
                    let zero_or_one = Pattern::ZeroOrOne(Box::new(last));
                    patterns.push(zero_or_one);
                } else {
                    patterns.push(Pattern::Literal("?".to_string()));
                }
            }
            _ => {
                literal_buffer.push(c);
            }
        }
    }

    // Flush any remaining literals in the buffer
    if !literal_buffer.is_empty() {
        patterns.push(Pattern::Literal(literal_buffer.clone()));
    }

    patterns
}

fn parse_group_pattern(group_pattern: &str) -> Vec<Pattern> {
    let mut alternatives = Vec::new();
    let mut current = String::new();
    let mut chars = group_pattern.chars().peekable();
    let mut depth = 0;

    while let Some(c) = chars.next() {
        match c {
            '(' => {
                depth += 1;
                current.push(c);
            }
            ')' => {
                depth -= 1;
                current.push(c);
            }
            '|' if depth == 0 => {
                let alternative_patterns = parse_pattern(&current);
                alternatives.push(Pattern::Group(alternative_patterns));
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        let alternative_patterns = parse_pattern(&current);
        alternatives.push(Pattern::Group(alternative_patterns));
    }

    vec![Pattern::Alternation(alternatives)]
}

fn match_class(pattern: &Pattern, input_chars: &mut Peekable<Chars>) -> bool {
    match pattern {
        Pattern::Digit => input_chars.next().map_or(false, |c| c.is_digit(10)),
        Pattern::Alphanumeric => input_chars.next().map_or(false, |c| c.is_alphanumeric()),
        Pattern::AnyChar => input_chars.next().is_some(),
        Pattern::CharGroup(group, is_negative) => {
            input_chars
                .next()
                .map_or(false, |c| group.contains(&c) != *is_negative)
        }
        Pattern::Start | Pattern::End => true, 
        _ => false,
    }
}

fn match_literal(literal: &str, input_chars: &mut Peekable<Chars>) -> bool {
    for lit_char in literal.chars() {
        match input_chars.next() {
            Some(input_char) if input_char == lit_char => continue,
            _ => return false,
        }
    }
    true
}

fn match_subpattern(
    pattern: &Pattern,
    input_chars: &mut Peekable<Chars>,
    captured_groups: &mut HashMap<usize, String>,
    current_group: Option<usize>,
) -> bool {
    let mut input_clone = input_chars.clone();
    let matched = match pattern {
        Pattern::Literal(ref literal) => match_literal(literal, &mut input_clone),
        Pattern::Digit | Pattern::Alphanumeric | Pattern::AnyChar | Pattern::CharGroup(_, _) => {
            match_class(pattern, &mut input_clone)
        }
        Pattern::Group(ref subpatterns) => {
            let group_num = captured_groups.len() + 1;
            if match_from_current_position(
                &mut input_clone,
                subpatterns,
                false,
                captured_groups,
            ) {
                let captured = extract_captured(input_chars, &input_clone);
                captured_groups.insert(group_num, captured);
                true
            } else {
                false
            }
        }
        Pattern::Alternation(ref alternatives) => {
            #[allow(unused_assignments)] // Suppress the unused_assignments warning for this block
            {
                for alternative in alternatives {
                    let mut clone = input_clone.clone();
                    let mut clone_captured = captured_groups.clone();
                    if match_subpattern(
                        alternative,
                        &mut clone,
                        &mut clone_captured,
                        current_group,
                    ) {
                        // If a match is found, update the input and captured groups
                        input_clone = clone;
                        *captured_groups = clone_captured;
                        return true;
                    }
                }
                false
            }
        }
        
        Pattern::BackReference(group_num) => {
            if let Some(captured) = captured_groups.get(group_num) {
                let mut temp_input = input_clone.clone();
                if match_literal(captured, &mut temp_input) {
                    *input_chars = temp_input;
                    return true;
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }
        _ => false, 
    };
    if matched {
        *input_chars = input_clone;
    }
    matched
}

fn match_from_current_position(
    input_chars: &mut Peekable<Chars>,
    patterns: &[Pattern],
    is_start: bool,
    captured_groups: &mut HashMap<usize, String>,
) -> bool {
    let mut input_clone = input_chars.clone();
    println!("Attempting to match from current position...");
    for (i, pattern) in patterns.iter().enumerate() {
        match pattern {
            Pattern::Literal(ref literal) => {
                println!("Matching Literal: '{}'", literal);
                if !match_literal(literal, &mut input_clone) {
                    println!("Literal '{}' did not match.", literal);
                    return false;
                }
            }
            Pattern::Start => {
                println!("Matching Start Anchor");
                if i != 0 || !is_start {
                    println!("Start anchor not at the beginning.");
                    return false;
                }
            }
            Pattern::End => {
                println!("Matching End Anchor");
                if i != patterns.len() - 1 || input_clone.peek().is_some() {
                    println!("End anchor does not match.");
                    return false;
                }
            }
            Pattern::OneOrMore(ref subpattern) => {
                println!("Matching OneOrMore for pattern: {:?}", subpattern);
                if !match_subpattern(
                    subpattern,
                    &mut input_clone,
                    captured_groups,
                    None,
                ) {
                    println!("OneOrMore subpattern did not match at least once.");
                    return false;
                }
                while match_subpattern(
                    subpattern,
                    &mut input_clone,
                    captured_groups,
                    None,
                ) {
                    println!("OneOrMore subpattern matched another instance.");
                }
            }
            Pattern::ZeroOrOne(ref subpattern) => {
                println!("Matching ZeroOrOne for pattern: {:?}", subpattern);
                if match_subpattern(
                    subpattern,
                    &mut input_clone,
                    captured_groups,
                    None,
                ) {
                    println!("ZeroOrOne subpattern matched once.");
                } else {
                    println!("ZeroOrOne subpattern did not match; proceeding without it.");
                }
            }
            Pattern::Group(ref subpatterns) => {
                println!("Matching Group");
                if !match_from_current_position(
                    &mut input_clone,
                    subpatterns,
                    false,
                    captured_groups,
                ) {
                    println!("Group did not match.");
                    return false;
                }
            }
            Pattern::Alternation(ref alternatives) => {
                println!("Matching Alternation: {:?}", alternatives);
                let mut alternation_matched = false;
                for alternative in alternatives {
                    let mut clone = input_clone.clone();
                    let mut clone_captured = captured_groups.clone();
                    if match_subpattern(
                        alternative,
                        &mut clone,
                        &mut clone_captured,
                        None,
                    ) {
                        input_clone = clone;
                        *captured_groups = clone_captured;
                        alternation_matched = true;
                        println!("Alternation alternative {:?} matched.", alternative);
                        break;
                    }
                }
                if !alternation_matched {
                    println!("No alternation alternatives matched.");
                    return false;
                }
            }
            Pattern::BackReference(group_num) => {
                println!("Matching BackReference: \\{}", group_num);
                if !match_subpattern(
                    pattern,
                    &mut input_clone,
                    captured_groups,
                    None,
                ) {
                    println!("BackReference \\{} did not match.", group_num);
                    return false;
                }
            }
            _ => {
                println!("Matching Class Pattern: {:?}", pattern);
                if !match_class(pattern, &mut input_clone) {
                    println!("Class pattern did not match.");
                    return false;
                }
            }
        }
    }
    *input_chars = input_clone;
    println!("Pattern matched successfully.");
    true
}

fn extract_captured(before: &Peekable<Chars>, after: &Peekable<Chars>) -> String {
    let before_str: String = before.clone().collect();
    let after_str: String = after.clone().collect();

    if before_str.len() >= after_str.len() {
        let captured_len = before_str.len() - after_str.len();
        before_str[..captured_len].to_string()
    } else {
        String::new()
    }
}

pub fn match_pattern(input_line: &str, pattern_str: &str) -> bool {
    println!("Input: '{}', Pattern: '{}'", input_line, pattern_str);
    let patterns = parse_pattern(pattern_str);
    let mut input_chars = input_line.chars().peekable();
    let mut captured_groups: HashMap<usize, String> = HashMap::new();

    let starts_with_anchor = matches!(patterns.first(), Some(Pattern::Start));
    let ends_with_anchor = matches!(patterns.last(), Some(Pattern::End));

    if starts_with_anchor && ends_with_anchor {
        println!("Pattern has both Start and End anchors.");
        return match_from_current_position(
            &mut input_chars,
            &patterns,
            true,
            &mut captured_groups,
        );
    } else if starts_with_anchor {
        println!("Pattern has Start anchor.");
        return match_from_current_position(
            &mut input_chars,
            &patterns,
            true,
            &mut captured_groups,
        );
    } else if ends_with_anchor {
        println!("Pattern has End anchor.");
        while input_chars.peek().is_some() {
            let mut clone = input_chars.clone();
            let mut clone_captured = captured_groups.clone();
            if match_from_current_position(
                &mut clone,
                &patterns,
                false,
                &mut clone_captured,
            ) && clone.peek().is_none()
            {
                println!("Pattern matched with End anchor.");
                return true;
            }
            input_chars.next();
        }
    } else {
        println!("Pattern has no anchors. Searching for pattern anywhere in the input.");
        while input_chars.peek().is_some() {
            let mut clone = input_chars.clone();
            let mut clone_captured = captured_groups.clone();
            if match_from_current_position(
                &mut clone,
                &patterns,
                false,
                &mut clone_captured,
            ) {
                println!("Pattern matched.");
                return true;
            }
            input_chars.next();
        }
    }

    println!("Pattern did not match.");
    false
}
