use crate::grammar;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    // Mots-clés
    Fn, Main, Log, For, To,
    // Identifiants / littéraux
    Ident(String),
    Number(String),   // entier décimal
    Str(String),      // "…"
    // Ponctuation / opérateurs
    LParen, RParen, LBrace, RBrace, Comma,
    Assign,
    // Fin
    Eof,
}

#[derive(Debug, Clone)]
pub struct LexError {
    pub message: String,
    pub at_byte: usize,
}
impl std::fmt::Display for LexError {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        write!(f, "{} (byte {})", self.message, self.at_byte)
    }
}
impl std::error::Error for LexError {}

pub struct Lexer<'a> {
    input: &'a str,
    bytes: &'a [u8],
    i: usize, // index byte courant
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, bytes: input.as_bytes(), i: 0 }
    }

    fn eof(&self) -> bool { self.i >= self.bytes.len() }
    fn peek(&self) -> Option<u8> { self.bytes.get(self.i).copied() }
    fn bump(&mut self) -> Option<u8> { let b=self.peek()?; self.i += 1; Some(b) }

    fn skip_ws(&mut self) {
        while let Some(b) = self.peek() {
            match b {
                b' ' | b'\t' | b'\n' | b'\r' => self.i += 1,
                _ => break,
            }
        }
    }

    fn starts_with(&self, s: &str) -> bool {
        self.input[self.i..].starts_with(s)
    }
    fn try_take(&mut self, s: &str) -> bool {
        if self.starts_with(s) { self.i += s.len(); true } else { false }
    }

    fn is_ident_start(b: u8) -> bool {
        (b'a'..=b'z').contains(&b) || (b'A'..=b'Z').contains(&b) || b == b'_'
    }
    fn is_ident_continue(b: u8) -> bool {
        Self::is_ident_start(b) || (b'0'..=b'9').contains(&b)
    }

    fn read_ident(&mut self) -> (&'a str, usize, usize) {
        let start = self.i;
        while let Some(b) = self.peek() {
            if Self::is_ident_continue(b) { self.i += 1; } else { break; }
        }
        ( &self.input[start..self.i], start, self.i )
    }

    fn read_number(&mut self) -> (&'a str, usize, usize) {
        let start = self.i;
        while let Some(b) = self.peek() {
            if (b'0'..=b'9').contains(&b) { self.i += 1; } else { break; }
        }
        (&self.input[start..self.i], start, self.i)
    }

    fn read_string(&mut self) -> Result<Token, LexError> {
        let start = self.i;
        self.bump(); // '"'
        let s = self.i;
        while let Some(b) = self.peek() {
            if b == b'"' {
                let out = &self.input[s..self.i];
                self.i += 1; // consume closing "
                return Ok(Token::Str(out.to_string()));
            }
            self.i += 1;
        }
        Err(LexError { message: "chaine non terminée".into(), at_byte: start })
    }

    /// Essaie l'opérateur d'affectation paramétrable (ASSIGN_LEXEME).
    fn try_assign(&mut self) -> Option<Token> {
        if self.try_take(grammar::ASSIGN_LEXEME) {
            Some(Token::Assign)
        } else {
            None
        }
    }

    /// Essaie la ponctuation paramétrable (toutes en &str).
    fn try_punct(&mut self) -> Option<Token> {
        if self.try_take(grammar::LPAREN)  { return Some(Token::LParen) }
        if self.try_take(grammar::RPAREN)  { return Some(Token::RParen) }
        if self.try_take(grammar::LBRACE)  { return Some(Token::LBrace) }
        if self.try_take(grammar::RBRACE)  { return Some(Token::RBrace) }
        if self.try_take(grammar::COMMA)   { return Some(Token::Comma) }
        None
    }

    pub fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_ws();
        if self.eof() { return Ok(Token::Eof) }

        // 1) opérateur d'affectation (supporte "=" ou ":=" selon grammar.rs)
        if let Some(tok) = self.try_assign() {
            return Ok(tok);
        }

        // 2) ponctuation
        if let Some(tok) = self.try_punct() {
            return Ok(tok);
        }

        // 3) littéral string
        if self.peek() == Some(b'"') {
            return self.read_string();
        }

        // 4) identifiant / mot-clé
        if let Some(b) = self.peek() {
            if Self::is_ident_start(b) {
                let (id, at, _) = self.read_ident();
                return Ok(match id {
                    x if x == grammar::KW_FN   => Token::Fn,
                    x if x == grammar::KW_MAIN => Token::Main,
                    x if x == grammar::KW_LOG  => Token::Log,
                    x if x == grammar::KW_FOR  => Token::For,
                    x if x == grammar::KW_TO   => Token::To,
                    _ => Token::Ident(id.to_string()),
                });
            }
            // 5) nombre décimal
            if (b'0'..=b'9').contains(&b) {
                let (n, _, _) = self.read_number();
                return Ok(Token::Number(n.to_string()));
            }
        }

        Err(LexError {
            message: format!("caractère inattendu: 0x{:02X}", self.peek().unwrap()),
            at_byte: self.i,
        })
    }
}
