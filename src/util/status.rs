use std::{
    fmt::{Debug, Display},
    ops::RangeInclusive,
    str::FromStr,
};

use thiserror::Error;

const VALID_STATUS_CODES: RangeInclusive<i32> = (u8::MIN as i32)..=(u8::MAX as i32);

/// A set of status codes which can be parsed from a string.
/// A range may be indicated using two dots (eg 1..3).
/// Different subpatterns may be seperated by a comma (eg 1,2,3..5).
/// Two dots are used instead of a hyphen so that negative status codes
/// may be represented; this leaves the door open to supporting platforms
/// with negative status codes (eg Windows) in the future, without breaking
/// backwards compatibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusCodePattern {
    codes: [bool; 256],
}

impl StatusCodePattern {
    pub fn contains(&self, code: i32) -> bool {
        self.codes[code as usize]
    }

    // Testing utils
    #[allow(unused)]
    pub fn with_range(mut self, range: RangeInclusive<i32>) -> Self {
        assert!(VALID_STATUS_CODES.contains(range.start()));
        assert!(VALID_STATUS_CODES.contains(range.end()));
        for code in range {
            self.codes[code as usize] = true;
        }

        self
    }
    #[allow(unused)]
    pub fn with_code(mut self, code: i32) -> Self {
        assert!(VALID_STATUS_CODES.contains(&code));
        self.codes[code as usize] = true;
        self
    }
    #[allow(unused)]
    pub fn only(code: i32) -> Self {
        Self::default().with_code(code)
    }
}
impl Default for StatusCodePattern {
    fn default() -> Self {
        Self {
            codes: [false; 256],
        }
    }
}

impl FromStr for StatusCodePattern {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        type K = InvalidTokenKind;
        let mut code_table = [false; 256];
        let mut numeric_characters: Option<RangeInclusive<usize>> = None;
        let mut range_begins: Option<i32> = None;
        let mut dots_in_a_row: usize = 0;
        let mut last_was_whitespace: bool = false;

        for (i, char) in s.chars().enumerate() {
            if char != '.' && dots_in_a_row == 1 {
                // Catch a single dot (1.3 instead of 1..3)
                return Err(ParsingError {
                    kind: K::Range,
                    input: s.to_string(),
                    idx: i,
                });
            }

            if char.is_whitespace() {
                // Do nothing
            } else if char.is_ascii_digit() {
                // Track a window of valid numeric characters
                if let Some(num) = numeric_characters.as_ref() {
                    if last_was_whitespace {
                        // Catch bad whitespace (1 2)
                        return Err(ParsingError {
                            kind: K::Number,
                            input: s.to_string(),
                            idx: i - 1,
                        });
                    }
                    numeric_characters = Some(*num.start()..=i)
                } else {
                    numeric_characters = Some(i..=i)
                }
            } else if char == '.' {
                // `..` indicates that current_characters are the start of a range
                dots_in_a_row += 1;
                if dots_in_a_row == 2 {
                    if last_was_whitespace {
                        // Catch bad whitespace (1. .2)
                        return Err(ParsingError {
                            kind: K::Range,
                            input: s.to_string(),
                            idx: i - 1,
                        });
                    } else {
                        // Do nothing
                    }
                } else if dots_in_a_row > 2 {
                    // Catch too many dots (1...2)
                    return Err(ParsingError {
                        kind: K::Range,
                        input: s.to_string(),
                        idx: i,
                    });
                } else if range_begins.is_some() {
                    // Catches broken ranges (1..2..3)
                    return Err(ParsingError {
                        kind: K::Range,
                        input: s.to_string(),
                        idx: i,
                    });
                } else if numeric_characters.is_none() {
                    // No beginning supplied (..1)
                    return Err(ParsingError {
                        kind: K::Range,
                        input: s.to_string(),
                        idx: i,
                    });
                } else {
                    // On the first dot, capture the code in the current window
                    if let Ok(code) = i32::from_str(&s[numeric_characters.take().unwrap()]) {
                        if VALID_STATUS_CODES.contains(&code) {
                            range_begins = Some(code);
                        } else {
                            return Err(ParsingError {
                                kind: K::Status,
                                input: s.to_string(),
                                idx: i - dots_in_a_row,
                            });
                        }
                    } else {
                        return Err(ParsingError {
                            kind: K::Number,
                            input: s.to_string(),
                            idx: i - dots_in_a_row,
                        });
                    }
                }
            } else if char == ',' {
                // Pop the subpattern we've been parsing
                if let Some(num) = numeric_characters.take() {
                    let code = match i32::from_str(&s[num]) {
                        Ok(c) if VALID_STATUS_CODES.contains(&c) => c,
                        Ok(_) => {
                            return Err(ParsingError {
                                kind: K::Status,
                                input: s.to_string(),
                                idx: i - 1,
                            });
                        }
                        Err(_) => {
                            return Err(ParsingError {
                                kind: K::Number,
                                input: s.to_string(),
                                idx: i - 1,
                            })
                        }
                    };

                    if let Some(begin) = range_begins.take() {
                        // Tolerate backwards ranges
                        let start = begin.min(code);
                        let end = begin.max(code);
                        for c in start..=end {
                            code_table[c as usize] = true;
                        }
                        dots_in_a_row = 0;
                    } else {
                        code_table[code as usize] = true;
                    }
                } else if range_begins.is_some() {
                    return Err(ParsingError {
                        kind: K::Range,
                        input: s.to_string(),
                        idx: i,
                    });
                }
            } else {
                return Err(ParsingError {
                    kind: K::Characters,
                    input: s.to_string(),
                    idx: i,
                });
            }

            last_was_whitespace = char.is_whitespace();
        }

        if let Some(num) = numeric_characters.take() {
            // Pop the last subpattern
            let code = match i32::from_str(&s[num]) {
                Ok(c) if VALID_STATUS_CODES.contains(&c) => c,
                Ok(_) => {
                    return Err(ParsingError {
                        kind: K::Status,
                        input: s.to_string(),
                        idx: s.len() - 1,
                    });
                }
                Err(_) => {
                    return Err(ParsingError {
                        kind: K::Number,
                        input: s.to_string(),
                        idx: s.len() - 1,
                    })
                }
            };

            if let Some(begin) = range_begins.take() {
                // Tolerate backwards ranges
                let start = begin.min(code);
                let end = begin.max(code);
                for c in start..=end {
                    code_table[c as usize] = true;
                }
            } else {
                code_table[code as usize] = true;
            }
        } else if range_begins.is_some() {
            // We started a range we never completed (eg `1..`)
            return Err(ParsingError {
                kind: K::Range,
                input: s.to_string(),
                idx: s.len() - 1,
            });
        }

        Ok(Self { codes: code_table })
    }
}

#[derive(Error, Clone, Debug, PartialEq, Eq)]
pub struct ParsingError {
    pub kind: InvalidTokenKind,
    pub input: String,
    pub idx: usize,
}
impl Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{kind}{newline}{input}{newline}{padding}^",
            newline = "\n  ",
            kind = self.kind,
            input = &self.input,
            padding = " ".repeat(self.idx),
        ))
    }
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidTokenKind {
    #[error("Invalid characters: must be digits, commas, periods, or whitespace.")]
    Characters,
    #[error("Statuses must be in the range [0, 255].")]
    Status,
    #[error("Range could not be understood.")]
    Range,
    #[error("Number could not be understood.")]
    Number,
}

#[cfg(test)]
mod test {
    use super::*;
    type K = InvalidTokenKind;

    #[test]
    fn parsing_single_status_code() {
        let s = StatusCodePattern::from_str("1").unwrap();
        assert_eq!(s, StatusCodePattern::only(1));
        assert!(s.contains(1));
        assert!(!s.contains(0));
    }

    #[test]
    fn parsing_several_status_codes() {
        let s = StatusCodePattern::from_str("1,2,3").unwrap();
        assert_eq!(s, StatusCodePattern::default().with_range(1..=3));
        assert!(s.contains(1));
        assert!(s.contains(2));
        assert!(s.contains(3));
        assert!(!s.contains(0));
    }

    #[test]
    fn parsing_status_code_range() {
        let s = StatusCodePattern::from_str("1..3").unwrap();
        assert_eq!(s, StatusCodePattern::default().with_range(1..=3));
        assert!(s.contains(1));
        assert!(s.contains(2));
        assert!(s.contains(3));
        assert!(!s.contains(0))
    }

    #[test]
    fn backwards_ranges_are_fixed() {
        let a = StatusCodePattern::from_str("1..10").unwrap();
        let b = StatusCodePattern::from_str("10..1").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn parsing_complex_status_code() {
        let s = StatusCodePattern::from_str("1..3,5,10..12").unwrap();
        assert_eq!(
            s,
            StatusCodePattern::default()
                .with_range(1..=3)
                .with_code(5)
                .with_range(10..=12)
        );
        assert!(s.contains(1));
        assert!(s.contains(2));
        assert!(s.contains(3));

        assert!(s.contains(5));

        assert!(s.contains(10));
        assert!(s.contains(11));
        assert!(s.contains(12));

        assert!(!s.contains(0))
    }

    #[test]
    fn parsing_status_codes_ignores_gratuitous_commas() {
        assert!(StatusCodePattern::from_str("1,").is_ok());
        assert!(StatusCodePattern::from_str(",1").is_ok());
        assert!(StatusCodePattern::from_str("1,2,,,,").is_ok());
        assert!(StatusCodePattern::from_str(",,,,1,2").is_ok());
        assert!(StatusCodePattern::from_str("1,,,,2").is_ok());
    }

    #[test]
    fn parsing_status_codes_fails_on_invalid_chars() {
        fn assert(s: &str) {
            let err = StatusCodePattern::from_str(s).err().unwrap();
            assert_eq!(err.kind, K::Characters, "Invalid error kind for \"{}\"", s);
            assert_eq!(err.idx, s.find('!').unwrap(), "Invalid index for \"{}\"", s);
        }

        assert("123!123");
        assert("1!2..3");
        assert("123!");
        assert("!123");
        assert("1,!123");
        assert("1..5,!123");
    }

    #[test]
    fn parsing_status_codes_fails_on_invalid_status_codes() {
        fn assert(s: &str) {
            const ERR_STR: &str = "256";
            let err = StatusCodePattern::from_str(s).err().unwrap();
            assert_eq!(err.kind, K::Status, "Invalid error kind for \"{}\"", s);
            assert_eq!(
                err.idx,
                s.find(ERR_STR).unwrap() + ERR_STR.len() - 1,
                "Invalid index for \"{}\"",
                s
            );
        }

        assert("256");
        assert("1,2,256");
        assert("1,256,2");
        assert("1..256");
        assert("256..1");
        assert("1,2,1..256");
        assert("1,1..256,2");
        assert("1..256,1,2");
        assert("1,2,256..1");
        assert("1,256..1,2");
        assert("256..1,1,2");
    }

    #[test]
    fn parsing_status_codes_fails_on_bad_ranges() {
        let err = StatusCodePattern::from_str("123..").err().unwrap();
        assert_eq!(err.kind, K::Range);
        assert_eq!(err.idx, 4);

        let err = StatusCodePattern::from_str("..123").err().unwrap();
        assert_eq!(err.kind, K::Range);
        assert_eq!(err.idx, 0);

        let err = StatusCodePattern::from_str("1..2..3").err().unwrap();
        assert_eq!(err.kind, K::Range);
        assert_eq!(err.idx, 4);

        let err = StatusCodePattern::from_str("123..,1").err().unwrap();
        assert_eq!(err.kind, K::Range);
        assert_eq!(err.idx, 5);

        let err = StatusCodePattern::from_str("1,123..").err().unwrap();
        assert_eq!(err.kind, K::Range);
        assert_eq!(err.idx, 6);

        let err = StatusCodePattern::from_str("..123,1").err().unwrap();
        assert_eq!(err.kind, K::Range);
        assert_eq!(err.idx, 0);

        let err = StatusCodePattern::from_str("1,..123").err().unwrap();
        assert_eq!(err.kind, K::Range);
        assert_eq!(err.idx, 2);

        let err = StatusCodePattern::from_str("1..2..3,1").err().unwrap();
        assert_eq!(err.kind, K::Range);
        assert_eq!(err.idx, 4);

        let err = StatusCodePattern::from_str("1,1..2..3").err().unwrap();
        assert_eq!(err.kind, K::Range);
        assert_eq!(err.idx, 6);
    }

    #[test]
    fn valid_whitespace_is_ignored() {
        let a = StatusCodePattern::from_str("1,2,3").unwrap();
        let b = StatusCodePattern::from_str("1, 2,\t3").unwrap();
        assert_eq!(a, b);

        let c = StatusCodePattern::from_str("\t\t   \t1, \t2,       3          ").unwrap();
        assert_eq!(a, c);
    }

    #[test]
    fn parsing_status_codes_catches_invalid_whitespace() {
        fn assert(s: &str, kind: InvalidTokenKind) {
            let err = StatusCodePattern::from_str(s).err().unwrap();
            assert_eq!(err.kind, kind, "Invalid error kind for \"{}\"", s);
            assert_eq!(err.idx, s.find(' ').unwrap(), "Invalid index for \"{}\"", s);
        }

        assert("1 2", K::Number);
        assert("5,1 2", K::Number);
        assert("1 2,5", K::Number);

        assert("1. .2", K::Range);
        assert("5,1. .2", K::Range);
        assert("1. .2,5", K::Range);
    }
}
