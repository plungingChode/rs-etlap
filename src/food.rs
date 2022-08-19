use crate::cursor::Cursor;
use serde::Serialize;
use TokenKind::*;

pub fn parse(input: &str) -> Vec<Food> {
    let tokens = tokenize(input);
    let food = as_food(tokens, input);
    food
}

#[derive(Debug, Serialize)]
pub struct Food {
    pub name: String,
    pub allergens: Option<String>,
}

#[derive(Debug)]
struct Span {
    kind: TokenKind,
    start: usize,
    len: usize,
}

#[derive(Debug)]
struct Token {
    kind: TokenKind,
    len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenKind {
    Name,
    Allergens,
    Garbage,
}

impl Span {
    pub fn string(&self, input: &str) -> String {
        // Unsafe indexing, since Chars::as_str().len() returns the
        // **byte** length, instead of the character length
        // See: Cursor::len_consumed()
        //
        // Also replace all newline characters, since that's what we use
        // to (try to) separate cell contents
        input[self.start..self.start + self.len]
            .trim()
            .to_owned()
            .replace("\n", "")
    }
}

fn as_food(tokens: impl Iterator<Item = Token>, input: &str) -> Vec<Food> {
    let mut start_idx = 0;
    let mut spans = tokens
        .map(move |tok| {
            let span = match tok.kind {
                Allergens | Name => Span {
                    kind: tok.kind,
                    len: tok.len,
                    start: start_idx,
                },
                _ => Span {
                    kind: tok.kind,
                    start: 0,
                    len: 0,
                },
            };
            start_idx += tok.len;
            span
        })
        .peekable();

    let mut food: Vec<Food> = vec![];
    if spans.peek().is_none() {
        return food;
    }

    while let Some(s) = spans.next() {
        // Allergens are handled strictly after a name was found,
        // Garbage tokens are skipped altogether
        if matches!(s.kind, Garbage | Allergens) {
            continue;
        }

        let name = s.string(input);
        let allergens = match spans.peek() {
            Some(a) if a.kind == Allergens => Some(a.string(input)),
            _ => None,
        };

        food.push(Food { name, allergens });
    }

    food
}

fn tokenize(text: &str) -> impl Iterator<Item = Token> + '_ {
    let mut cursor = Cursor::new(text);
    std::iter::from_fn(move || {
        if cursor.is_eof() {
            None
        } else {
            cursor.reset_len_consumed();
            Some(cursor.next_token())
        }
    })
}

impl Cursor<'_> {
    fn next_token(&mut self) -> Token {
        let first_char = self.get_next().unwrap();
        let token_kind = match first_char {
            // Whitespace
            c if c.is_whitespace() => self.whitespace(),
            // Quantifier
            c if c.is_numeric() => self.quantifier(),
            // Standalone all caps unit or qualifier (e.g DIÉTÁS)
            c if c.is_uppercase() && self.peek_first().is_uppercase() => {
                self.get_while(|c| c.is_uppercase());
                TokenKind::Garbage
            }
            // Name
            c if c.is_uppercase() => self.name(),
            // Allergen list
            '(' => self.allergens(),
            // Anything else we don't care about
            _ => TokenKind::Garbage,
        };

        let token_len = self.len_consumed();
        Token {
            kind: token_kind,
            len: token_len,
        }
    }

    /// Name is a capitalized string. It may have supplementary information
    /// after it, inside parentheses.
    /// 
    /// # Warning
    /// Doesn't actually cover *every possible* food name, since the way people 
    /// input the name of the additional information is not regulated. This way
    /// we end up with names like "Májkrém Hamé" (interpreted as two distinct 
    /// food items, "Májkrém" and "Hamé") or "Gríz(30 g)", which is supposed to
    ///  be a quantifier.
    fn name(&mut self) -> TokenKind {
        // Read food name
        self.get_while(|c| c != '*' && (c.is_lowercase() || c.is_whitespace()));

        // Read until we hit a "([0-9]" part, which must be the allergen list.
        // Any other text in parens *should* be part of the food's name (e.g.
        // its manufacturer)
        if self.peek_first() == '(' && !self.peek_second().is_numeric() {
            self.get_while(|c| c != ')');
            self.get_next();
        }

        TokenKind::Name
    }

    fn whitespace(&mut self) -> TokenKind {
        self.get_while(|c| c.is_whitespace());
        TokenKind::Garbage
    }

    /// Quantifier is a number followed by all lowercase or all uppercase 
    /// unit qualifier.
    fn quantifier(&mut self) -> TokenKind {
        // Read the quantifier (e.g. 15 dkg). Lasts until we
        // hit the allergen list: "([0-9]" or an all caps unit
        self.get_while(|c| !c.is_uppercase() && (c != '(' || c.is_whitespace()));

        // Read ALLCAPS units (of course they exist...)
        if self.peek_first().is_uppercase() && self.peek_second().is_uppercase() {
            self.get_while(|c| c.is_uppercase());
        }

        TokenKind::Garbage
    }

    /// Allergens are a paretheses-enclosed list of numbers
    fn allergens(&mut self) -> TokenKind {
        self.get_while(|c| c != ')');
        self.get_next();
        TokenKind::Allergens
    }
}
