use crate::lexer::{Lexer, Token, LexError};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Program {
    pub logs: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ParseError {
    Lex(LexError),
    Unexpected {
        at_byte: usize,
        found: Token,
        expected: &'static str,
    },
    #[allow(dead_code)]
    EofWhile(&'static str),
}

impl From<LexError> for ParseError {
    fn from(e: LexError) -> Self { ParseError::Lex(e) }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Lex(e) => write!(f, "Lexer: {e}"),
            ParseError::Unexpected { at_byte, found, expected } =>
                write!(f, "Token inattendu à {at_byte}: {:?} (attendu: {expected})", found),
            ParseError::EofWhile(ctx) =>
                write!(f, "Fin de fichier inattendue en lisant {ctx}"),
        }
    }
}
impl std::error::Error for ParseError {}

pub struct Parser<'a> {
    lx: Lexer<'a>,
    cur: Token,        // lookahead courant
    cur_byte: usize,   // position approx (octet) du début de cur
}

impl<'a> Parser<'a> {
    pub fn new(mut lx: Lexer<'a>) -> Result<Self, ParseError> {
        // on “avance” une fois pour remplir cur
        let cur = lx.next_token()?;
        Ok(Parser { lx, cur, cur_byte: 0 })
    }

    fn bump(&mut self) -> Result<(), ParseError> {
        // NB: notre lexer minimal ne donne pas d'offset ; on laisse 0.
        self.cur = self.lx.next_token()?;
        Ok(())
    }

    fn expect_exact(&mut self, want: Token, expected: &'static str) -> Result<(), ParseError> {
        if std::mem::discriminant(&self.cur) == std::mem::discriminant(&want) {
            self.bump()?;
            Ok(())
        } else {
            Err(ParseError::Unexpected {
                at_byte: self.cur_byte,
                found: self.cur.clone(),
                expected,
            })
        }
    }

    fn expect_string(&mut self) -> Result<String, ParseError> {
        if let Token::Str(s) = &self.cur {
            let out = s.clone();
            self.bump()?;
            Ok(out)
        } else {
            Err(ParseError::Unexpected {
                at_byte: self.cur_byte,
                found: self.cur.clone(),
                expected: "une chaîne \"...\"",
            })
        }
    }

    fn is(&self, t: &Token) -> bool {
        std::mem::discriminant(&self.cur) == std::mem::discriminant(t)
    }

    /// Parse le programme complet.
    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        // fn main() { ... }
        self.expect_exact(Token::Fn, "`fn`")?;
        self.expect_exact(Token::Main, "`main`")?;
        self.expect_exact(Token::LParen, "`(`")?;
        self.expect_exact(Token::RParen, "`)`")?;
        self.expect_exact(Token::LBrace, "`{`")?;

        let mut logs = Vec::new();

        // { log("...") log("...") }
        while !self.is(&Token::RBrace) {
            // Statement := log("(" String ")")
            self.expect_exact(Token::Log, "`log`")?;
            self.expect_exact(Token::LParen, "`(`")?;
            let s = self.expect_string()?;
            self.expect_exact(Token::RParen, "`)`")?;
            logs.push(s);
        }

        self.expect_exact(Token::RBrace, "`}`")?;

        // On attend Eof
        self.expect_exact(Token::Eof, "fin de fichier")?;

        Ok(Program { logs })
    }
}
