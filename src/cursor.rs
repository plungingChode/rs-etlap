use std::str::Chars;

const EOF: char = '\0';

pub struct Cursor<'a> {
    input_len: usize,
    chars: Chars<'a>
}

impl<'a> Cursor<'a> {
    pub fn new(input: &'a str) -> Cursor<'a> {
        Cursor { input_len: input.len(), chars: input.chars() }
    }

    /// Peek the first character from the input stream.
    pub fn peek_first(&self) -> char {
        self.chars.clone().nth(0).unwrap_or(EOF)
    }

    /// Peek the second character from the input stream.
    pub fn peek_second(&self) -> char {
        self.chars.clone().nth(1).unwrap_or(EOF)
    }

    /// Check if the input has ran out.
    pub fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    /// Move the cursor to the next character.
    pub fn get_next(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        Some(c)
    }

    /// Move the cursor while the predicate returns true or the input runs out.
    pub fn get_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while predicate(self.peek_first()) && !self.is_eof() {
            self.get_next();
        }
    }

    pub fn len_consumed(&self) -> usize {
        self.input_len - self.chars.as_str().len()
    }

    pub fn reset_len_consumed(&mut self) {
        self.input_len = self.chars.as_str().len()
    }
}

