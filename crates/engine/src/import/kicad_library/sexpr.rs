//! Minimal first-party S-expression reader, independent of source indentation.
use super::{Result, refusal};

#[derive(Debug, Clone)]
pub(super) enum Node {
    Atom(String),
    List(Vec<Node>),
}

impl Node {
    pub fn atom(&self, index: usize) -> Option<&str> {
        match self.items().get(index)? {
            Self::Atom(value) => Some(value),
            Self::List(_) => None,
        }
    }

    pub fn required_atom(&self, index: usize) -> Result<&str> {
        self.atom(index).ok_or_else(|| {
            refusal(format!(
                "missing value {index} in {}",
                self.tag().unwrap_or("list")
            ))
        })
    }

    pub fn tag(&self) -> Option<&str> {
        self.atom(0)
    }

    pub fn items(&self) -> &[Node] {
        match self {
            Self::List(items) => items,
            Self::Atom(_) => &[],
        }
    }

    pub fn children<'a>(&'a self, tag: &'a str) -> impl Iterator<Item = &'a Node> {
        self.items()
            .iter()
            .filter(move |node| node.tag() == Some(tag))
    }

    pub fn child(&self, tag: &str) -> Option<&Node> {
        self.items().iter().find(|node| node.tag() == Some(tag))
    }

    pub fn required_child(&self, tag: &str) -> Result<&Node> {
        self.child(tag)
            .ok_or_else(|| refusal(format!("missing {tag} in {}", self.tag().unwrap_or("list"))))
    }

    pub fn flag(&self, tag: &str) -> bool {
        self.items()
            .iter()
            .any(|n| matches!(n, Self::Atom(s) if s == tag))
            || self.child(tag).is_some_and(|n| n.atom(1) != Some("no"))
    }
}

pub(super) fn parse(input: &str) -> Result<Node> {
    let mut reader = Reader {
        chars: input.chars().peekable(),
    };
    let root = reader.node(0)?;
    reader.whitespace();
    if reader.chars.next().is_some() {
        return Err(refusal("trailing source content"));
    }
    Ok(root)
}

struct Reader<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl Reader<'_> {
    fn whitespace(&mut self) {
        while self.chars.peek().is_some_and(|c| c.is_whitespace()) {
            self.chars.next();
        }
    }

    fn node(&mut self, depth: usize) -> Result<Node> {
        if depth > 128 {
            return Err(refusal("S-expression nesting exceeds 128"));
        }
        self.whitespace();
        match self
            .chars
            .next()
            .ok_or_else(|| refusal("unexpected end of source"))?
        {
            '(' => {
                let mut children = Vec::new();
                loop {
                    self.whitespace();
                    match self.chars.peek() {
                        Some(')') => {
                            self.chars.next();
                            break;
                        }
                        None => return Err(refusal("unclosed S-expression")),
                        _ => children.push(self.node(depth + 1)?),
                    }
                }
                Ok(Node::List(children))
            }
            ')' => Err(refusal("unexpected closing parenthesis")),
            '"' => {
                let mut value = String::new();
                loop {
                    match self.chars.next() {
                        Some('"') => break,
                        Some('\\') => match self.chars.next() {
                            Some('n') => value.push('\n'),
                            Some('r') => value.push('\r'),
                            Some('t') => value.push('\t'),
                            Some('"') => value.push('"'),
                            Some('\\') => value.push('\\'),
                            _ => return Err(refusal("unsupported string escape")),
                        },
                        Some(c) => value.push(c),
                        None => return Err(refusal("unclosed quoted string")),
                    }
                }
                Ok(Node::Atom(value))
            }
            first => {
                let mut value = String::from(first);
                while self
                    .chars
                    .peek()
                    .is_some_and(|c| !c.is_whitespace() && *c != '(' && *c != ')')
                {
                    value.push(self.chars.next().expect("peeked character"));
                }
                Ok(Node::Atom(value))
            }
        }
    }
}
