//! Bounded TS-literal parser. Keeps source order and duplicate properties.
//! It deliberately supports the literal/recovery surface exercised by json-ts;
//! arbitrary TypeScript programs are outside the documented native input contract.
use crate::{names, MAX_DEPTH, MAX_NODES};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Object,
    Array,
    Null,
    True,
    False,
    String,
    Number,
}
#[derive(Clone, Debug)]
pub(crate) struct Node {
    pub name: String,
    pub kind: Kind,
    pub candidate: bool,
    pub body: Vec<Node>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum Token {
    Text(String),
    Number(String),
    Word(String),
    P(char),
    Template,
    End,
}
struct Parser {
    tokens: Vec<Token>,
    at: usize,
    nodes: usize,
}
fn mapped(u: u32) -> char {
    char::from_u32(if (0xd800..=0xdfff).contains(&u) {
        u + 0xf0000
    } else {
        u
    })
    .unwrap_or('\0')
}
fn lex(input: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = input.chars().collect();
    let mut at = 0;
    let mut out = Vec::new();
    while at < chars.len() {
        if out.len() >= MAX_NODES * 12 {
            return Err("Input exceeds token limit".into());
        }
        let c = chars[at];
        at += 1;
        if c.is_whitespace() || c == '\u{feff}' {
            continue;
        }
        if c == '/' && chars.get(at) == Some(&'/') {
            while at < chars.len() && !matches!(chars[at], '\n' | '\r' | '\u{2028}' | '\u{2029}') {
                at += 1;
            }
            continue;
        }
        if c == '/' && chars.get(at) == Some(&'*') {
            at += 1;
            while at < chars.len() {
                if chars[at] == '*' && chars.get(at + 1) == Some(&'/') {
                    at += 2;
                    break;
                }
                at += 1;
            }
            continue;
        }
        if matches!(c, '\'' | '"' | '`') {
            let mut s = String::new();
            while at < chars.len() {
                let ch = chars[at];
                at += 1;
                if ch == c {
                    break;
                }
                if matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}') && c != '`' {
                    at -= 1;
                    break;
                }
                if ch != '\\' {
                    s.push_str(&names::units(&ch.to_string()));
                    continue;
                }
                let Some(&e) = chars.get(at) else { break };
                at += 1;
                match e {
                    'n' => s.push('\n'),
                    'r' => s.push('\r'),
                    't' => s.push('\t'),
                    'b' => s.push('\u{8}'),
                    'f' => s.push('\u{c}'),
                    'v' => s.push('\u{b}'),
                    '\n' => {}
                    '\r' => {
                        if chars.get(at) == Some(&'\n') {
                            at += 1;
                        }
                    }
                    'u' | 'x' => {
                        let brace = e == 'u' && chars.get(at) == Some(&'{');
                        if brace {
                            at += 1;
                        }
                        let mut hex = String::new();
                        let count = if brace {
                            8
                        } else if e == 'u' {
                            4
                        } else {
                            2
                        };
                        while hex.len() < count
                            && chars.get(at).is_some_and(|c| c.is_ascii_hexdigit())
                        {
                            hex.push(chars[at]);
                            at += 1;
                        }
                        if brace && chars.get(at) == Some(&'}') {
                            at += 1;
                        }
                        if let Ok(cp) = u32::from_str_radix(&hex, 16) {
                            if cp > 0xffff {
                                if let Some(c) = char::from_u32(cp) {
                                    s.push_str(&names::units(&c.to_string()));
                                }
                            } else {
                                s.push(mapped(cp));
                            }
                        } else {
                            s.push(e);
                            s.push_str(&hex);
                        }
                    }
                    '0'..='7' => {
                        let mut num = e.to_digit(8).unwrap_or(0);
                        let count = if e <= '3' { 2 } else { 1 };
                        for _ in 0..count {
                            if let Some(x) = chars.get(at).and_then(|x| x.to_digit(8)) {
                                num = num * 8 + x;
                                at += 1;
                            } else {
                                break;
                            }
                        }
                        s.push(mapped(num));
                    }
                    _ => s.push_str(&names::units(&e.to_string())),
                }
            }
            out.push(if c == '`' {
                Token::Template
            } else {
                Token::Text(s)
            });
            continue;
        }
        if c.is_ascii_digit() || (c == '.' && chars.get(at).is_some_and(char::is_ascii_digit)) {
            let start = at - 1;
            while at < chars.len()
                && (chars[at].is_ascii_alphanumeric()
                    || chars[at] == '.'
                    || (matches!(chars[at], '+' | '-') && matches!(chars[at - 1], 'e' | 'E')))
            {
                at += 1;
            }
            let raw: String = chars[start..at].iter().collect();
            out.push(Token::Number(normalize_number(&raw)));
            continue;
        }
        if c.is_alphabetic() || matches!(c, '_' | '$' | '\\') || u32::from(c) > 127 {
            let start = at - 1;
            while at < chars.len()
                && (chars[at].is_alphanumeric()
                    || matches!(chars[at], '_' | '$' | '\\')
                    || u32::from(chars[at]) > 127)
            {
                at += 1;
            }
            out.push(Token::Word(names::units(
                &chars[start..at].iter().collect::<String>(),
            )));
            continue;
        }
        out.push(Token::P(c));
    }
    out.push(Token::End);
    Ok(out)
}
fn normalize_number(raw: &str) -> String {
    // Only property names observe numeric values; TypeScript's scanner normalizes them.
    let number = if raw.starts_with("0x") || raw.starts_with("0X") {
        u64::from_str_radix(&raw[2..], 16).ok().map(|n| n as f64)
    } else if raw.starts_with("0b") || raw.starts_with("0B") {
        u64::from_str_radix(&raw[2..], 2).ok().map(|n| n as f64)
    } else if raw.starts_with("0o") || raw.starts_with("0O") {
        u64::from_str_radix(&raw[2..], 8).ok().map(|n| n as f64)
    } else {
        let end = raw
            .find(|c: char| !c.is_ascii_digit() && !".eE+-".contains(c))
            .unwrap_or(raw.len());
        raw[..end].parse::<f64>().ok()
    };
    number
        .map(|n| {
            if n.is_infinite() {
                "Infinity".into()
            } else {
                ryu_js::Buffer::new().format(n).into()
            }
        })
        .unwrap_or_else(|| raw.into())
}
impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.at.min(self.tokens.len() - 1)]
    }
    fn take(&mut self) -> Token {
        let t = self.peek().clone();
        if t != Token::End {
            self.at += 1;
        }
        t
    }
    fn eat(&mut self, c: char) -> bool {
        if self.peek() == &Token::P(c) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn expression(&mut self, depth: usize) -> Result<Option<Node>, String> {
        if depth > MAX_DEPTH {
            return Err("Input exceeds nesting limit (64)".into());
        }
        self.nodes += 1;
        if self.nodes > MAX_NODES {
            return Err("Input exceeds node limit (20000)".into());
        }
        let mut node = match self.take() {
            Token::Text(_) => Some(Node {
                name: String::new(),
                kind: Kind::String,
                candidate: false,
                body: vec![],
            }),
            Token::Number(_) => Some(Node {
                name: String::new(),
                kind: Kind::Number,
                candidate: false,
                body: vec![],
            }),
            Token::Word(s) if ["null", "true", "false"].contains(&s.as_str()) => Some(Node {
                name: String::new(),
                kind: match s.as_str() {
                    "null" => Kind::Null,
                    "true" => Kind::True,
                    _ => Kind::False,
                },
                candidate: false,
                body: vec![],
            }),
            Token::P('{') => Some(self.object(depth)?),
            Token::P('[') => {
                let mut body = Vec::new();
                while !matches!(self.peek(), Token::End | Token::P(']')) {
                    let before = self.at;
                    if self.eat(',') {
                        continue;
                    }
                    if let Some(n) = self.expression(depth + 1)? {
                        body.push(n);
                    }
                    self.eat(',');
                    if self.at == before {
                        self.take();
                    }
                }
                self.eat(']');
                Some(Node {
                    name: String::new(),
                    kind: Kind::Array,
                    candidate: false,
                    body,
                })
            }
            Token::P('+' | '-' | '!' | '~') => {
                let _ = self.expression(depth + 1)?;
                Some(Node {
                    name: String::new(),
                    kind: Kind::Number,
                    candidate: false,
                    body: vec![],
                })
            }
            Token::P('(') => {
                self.skip_balanced(')', depth + 1)?;
                None
            }
            Token::P('}' | ']' | ',' | ';') | Token::End => {
                if self.at > 0 {
                    self.at -= 1;
                }
                None
            }
            _ => None,
        };
        // Expressions rather than literals are ignored by json-ts. Calls, indexing,
        // binary/conditional expressions and templates never execute here.
        loop {
            if self.eat('(') {
                self.skip_balanced(')', depth + 1)?;
                node = None;
                continue;
            }
            if self.eat('[') {
                self.skip_balanced(']', depth + 1)?;
                node = None;
                continue;
            }
            if self.eat('.') {
                self.take();
                node = None;
                continue;
            }
            if matches!(
                self.peek(),
                Token::P(
                    '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' | '<' | '>' | '=' | '?' | ':'
                )
            ) || matches!(self.peek(),Token::Word(s) if ["as","in","instanceof"].contains(&s.as_str()))
            {
                let colon = self.peek() == &Token::P(':');
                if colon {
                    break;
                }
                self.take();
                let _ = self.expression(depth + 1)?;
                node = None;
                continue;
            }
            break;
        }
        Ok(node)
    }
    fn skip_balanced(&mut self, end: char, depth: usize) -> Result<(), String> {
        if depth > MAX_DEPTH {
            return Err("Input exceeds nesting limit (64)".into());
        }
        while !matches!(self.peek(), Token::End) && !self.eat(end) {
            match self.take() {
                Token::P('(') => self.skip_balanced(')', depth + 1)?,
                Token::P('[') => self.skip_balanced(']', depth + 1)?,
                Token::P('{') => self.skip_balanced('}', depth + 1)?,
                _ => {}
            }
        }
        Ok(())
    }
    fn object(&mut self, depth: usize) -> Result<Node, String> {
        let mut body = Vec::new();
        while !matches!(self.peek(), Token::End | Token::P('}')) {
            let before = self.at;
            let name = match self.take() {
                Token::Text(s) | Token::Word(s) | Token::Number(s) => Some(s),
                Token::P('[') => {
                    self.skip_balanced(']', depth + 1)?;
                    if self.eat(':') && self.expression(depth + 1)?.is_some() {
                        return Err("Invalid JSON".into());
                    }
                    None
                }
                Token::P('(') => {
                    self.skip_balanced(')', depth + 1)?;
                    None
                }
                _ => None,
            };
            if let Some(name) = name {
                if self.eat(':') || matches!(self.peek(), Token::Text(_)) {
                    if let Some(mut n) = self.expression(depth + 1)? {
                        n.candidate = n.kind == Kind::Object && !names::needs_quotes(&name)?;
                        n.name = name;
                        body.push(n);
                    }
                } else if self.eat('(') {
                    self.skip_balanced(')', depth + 1)?;
                    if self.eat('{') {
                        self.skip_balanced('}', depth + 1)?;
                    }
                }
            }
            self.eat(',');
            self.eat(';');
            if self.at == before {
                self.take();
            }
        }
        self.eat('}');
        Ok(Node {
            name: String::new(),
            kind: Kind::Object,
            candidate: true,
            body,
        })
    }
}
pub(crate) fn parse(input: &str) -> Result<Node, String> {
    let mut parser = Parser {
        tokens: lex(input)?,
        at: 0,
        nodes: 0,
    };
    let prefix = matches!(parser.peek(), Token::P('+' | '-' | '!' | '~'));
    let node = parser.expression(0)?.ok_or("Invalid JSON")?;
    if prefix {
        return Err("Invalid JSON".into());
    }
    Ok(node)
}
