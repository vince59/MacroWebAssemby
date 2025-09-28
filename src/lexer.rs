use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Fn,
    Main,
    Log,
    LParen,   // (
    RParen,   // )
    LBrace,   // {
    RBrace,   // }
    Str(String),
    Eof,
}

#[derive(Debug, Clone)]
pub struct LexError {
    pub message: String,
    pub at_byte: usize,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (à l’octet {})", self.message, self.at_byte)
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
    fn bump(&mut self) -> Option<u8> { let b = self.peek()?; self.i += 1; Some(b) }

    fn skip_ws(&mut self) {
        while let Some(b) = self.peek() {
            match b {
                b' ' | b'\t' | b'\n' | b'\r' => { self.i += 1; }
                _ => break,
            }
        }
    }

    fn is_ident_start(b: u8) -> bool {
        (b'A'..=b'Z').contains(&b) || (b'a'..=b'z').contains(&b) || b == b'_'
    }
    fn is_ident_continue(b: u8) -> bool {
        Self::is_ident_start(b) || (b'0'..=b'9').contains(&b)
    }

    fn read_ident(&mut self) -> (&'a str, usize, usize) {
        let start = self.i;
        while let Some(b) = self.peek() {
            if Self::is_ident_continue(b) { self.i += 1; } else { break; }
        }
        let end = self.i;
        (&self.input[start..end], start, end)
    }

    fn read_string(&mut self) -> Result<Token, LexError> {
        // précondition: le guillemet d'ouverture '"' est à consommer maintenant
        let start_quote = self.i;
        self.bump(); // consume opening "
        let start = self.i;
        while let Some(b) = self.peek() {
            if b == b'"' {
                let s = &self.input[start..self.i];
                self.i += 1; // consume closing "
                return Ok(Token::Str(s.to_string()));
            }
            // (version ultra-simple : pas d'échappements, pas de \")
            self.i += 1;
        }
        Err(LexError {
            message: "chaine non terminée (\" manquant)".into(),
            at_byte: start_quote,
        })
    }

    /// Renvoie le prochain token (ou Eof).
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_ws();
        if self.eof() { return Ok(Token::Eof); }

        match self.peek().unwrap() {
            b'(' => { self.i += 1; Ok(Token::LParen) }
            b')' => { self.i += 1; Ok(Token::RParen) }
            b'{' => { self.i += 1; Ok(Token::LBrace) }
            b'}' => { self.i += 1; Ok(Token::RBrace) }
            b'"' => self.read_string(),

            b if Self::is_ident_start(b) => {
                let (ident, at, _) = self.read_ident();
                match ident {
                    "fn"   => Ok(Token::Fn),
                    "main" => Ok(Token::Main),
                    "log"  => Ok(Token::Log),
                    _ => Err(LexError {
                        message: format!("identifiant inconnu: `{ident}` (seuls: fn, main, log)"),
                        at_byte: at,
                    }),
                }
            }

            other => Err(LexError {
                message: format!("caractere inattendu: 0x{other:02X}"),
                at_byte: self.i,
            }),
        }
    }
}
