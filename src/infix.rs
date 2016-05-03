use tagmap::MatchRule;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct ParseError;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unknown error.")
    }
}

/// Parse infix boolean algebra into a rule
pub fn parse_infix(text: &str) -> Result<MatchRule<String>, ParseError> {
    use tagmap::MatchRule::*;
    use self::Token::*;
    let mut rules = Vec::new();
    let tokens = tokenize(text);
    let mut not = false;
    for t in tokens {
        match t {
            Tag(ref s) => {
                let rule = if not {
                    not = false;
                    NotTags(vec![s.clone()])
                } else {
                    Tags(vec![s.clone()])
                };
                rules.push(rule);
            }
            PrefixNot => not = true,
            _ => {}
        }
    }
    Ok(Rules(rules))
}

#[derive(Debug, PartialEq)]
enum Token {
    Lparen,
    Rparen,
    PrefixNot,
    InfixAnd,
    InfixOr,
    Tag(String),
}

fn tokenize(text: &str) -> Vec<Token> {
    use self::Token::*;
    let mut tokens = Vec::new();
    let mut tag = String::new();
    for c in text.chars() {
        match c {
            '(' => tokens.push(Lparen),
            ')' => tokens.push(Rparen),
            '!' => tokens.push(PrefixNot),
            '&' => tokens.push(InfixAnd),
            '|' => tokens.push(InfixOr),
            _ if c.is_whitespace() => {
                if !tag.is_empty() {
                    tokens.push(Tag(tag.clone()));
                    tag.clear();
                }
            }
            _ => {
                tag.push(c);
            }
        }
    }
    if !tag.is_empty() {
        tokens.push(Tag(tag.clone()));
        tag.clear();
    }
    tokens
}

#[test]
fn test_tokenize() {
    use self::Token::*;
    assert_eq!(tokenize("foo !bar"),
               vec![Tag("foo".into()), PrefixNot, Tag("bar".into())]);
}

#[test]
fn test_parse() {
    use tagmap::MatchRule::*;
    assert_eq!(parse_infix("foo !bar"),
               Ok(Rules(vec![Tags(vec!["foo".into()]), NotTags(vec!["bar".into()])])));
}
