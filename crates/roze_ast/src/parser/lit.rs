use roze_lexer::Token;
use roze_span::Symbol;

use crate::{Lit, parser::Parser};

impl Parser<'_> {
    pub(super) fn parse_str_lit(&mut self, t: &Token) -> Lit {
        let s = &self.s[t];

        let bytes = s.as_bytes();
        if bytes.len() < 2 // the length must be sufficient, at least have 2 quotes
        || !matches!(bytes[0], b'\'' | b'"') // first character must be quote
        // both are the same quotes
        || bytes[0] != bytes.last().copied().unwrap()
        {
            panic!("Expected {s} str lit to start and end with the same quote.");
        }

        // intern inner data and return lit
        let s = &s[1..(s.len() - 1)];
        let str_sym = Symbol::intern(s);

        Lit::Str(str_sym)
    }

    pub(super) fn parse_num_lit(&mut self, t: &Token) -> Lit {
        let s = &self.s[t];
        let (prefix_len, radix) = prefix_and_radix(s.as_bytes());

        let mut chars = s[prefix_len..].chars().peekable();
        while let Some(c) = chars.next() {
            if c == '_' || c.is_digit(radix) {
                continue;
            }

            if radix == 10 && c == '.' {
                let mut has_digits = false;

                while let Some(c) = chars.next()
                    && !matches!(c, 'e' | 'E')
                {
                    if !c.is_ascii_digit() && c != '_' {
                        panic!("non decimal character '{c}' in number fraction: {s}");
                    }

                    has_digits = true;
                }

                if !has_digits {
                    panic!("malformed number: {s}, the fraction is empty")
                }
            }

            if radix == 10 && matches!(c, 'e' | 'E') {
                if matches!(chars.peek(), Some('+' | '-')) {
                    chars.next();
                }

                let mut has_digits = false;

                for c in chars {
                    if !c.is_ascii_digit() && c != '_' {
                        panic!("non decimal character '{c}' in number's exponent: {s}");
                    }

                    has_digits = true;
                }

                if !has_digits {
                    panic!("malformed number: {s}, the exponent is empty");
                }

                break; // the end of the number
            }
        }

        // return the symbol of the number literal
        let sym = Symbol::intern(s);
        Lit::Num(sym)
    }
}

fn prefix_and_radix(bytes: &[u8]) -> (usize, u32) {
    match bytes {
        [b'0', b'b' | b'B', ..] => (2, 2),
        [b'0', b'o' | b'O', ..] => (2, 8),
        [b'0', b'x' | b'X', ..] => (2, 16),
        _ => (0, 10),
    }
}
