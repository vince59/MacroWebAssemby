use crate::lexer::{Lexer, Token, LexError};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Program { pub stmts: Vec<Stmt> }

#[derive(Debug, Clone)]
pub enum Stmt {
    Log(Vec<Expr>),
    For { name:String, start:i32, end:i32, body:Vec<Stmt> },
}

#[derive(Debug, Clone)]
pub enum Expr { Str(String), Var(String), Int(i32) }

#[derive(Debug, Clone)]
pub enum ParseError {
    Lex(LexError),
    Unexpected { found: Token, expected: &'static str },
    IntOverflow(String),
}
impl From<LexError> for ParseError { fn from(e:LexError)->Self{ Self::Lex(e) } }
impl fmt::Display for ParseError{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        match self{
            Self::Lex(e)=>write!(f,"Lexer: {e}"),
            Self::Unexpected{found,expected}=>write!(f,"Attendu {expected}, trouvé {:?}",found),
            Self::IntOverflow(s)=>write!(f,"Entier hors plage i32: {s}"),
        }
    }
}
impl std::error::Error for ParseError {}

pub struct Parser<'a> {
    lx: Lexer<'a>,
    cur: Token,
}
impl<'a> Parser<'a> {
    pub fn new(mut lx:Lexer<'a>) -> Result<Self, ParseError> {
        let cur = lx.next_token()?;
        Ok(Self{ lx, cur })
    }
    fn bump(&mut self)->Result<(),ParseError>{ self.cur = self.lx.next_token()?; Ok(()) }
    fn expect(&mut self, want: Token, name:&'static str)->Result<(),ParseError>{
        if std::mem::discriminant(&self.cur)==std::mem::discriminant(&want){ self.bump()?; Ok(()) }
        else { Err(ParseError::Unexpected{ found:self.cur.clone(), expected:name }) }
    }
    fn parse_number_i32(&mut self)->Result<i32,ParseError>{
        if let Token::Number(s) = &self.cur {
            let v = s.parse::<i64>().map_err(|_|ParseError::IntOverflow(s.clone()))?;
            if v < i32::MIN as i64 || v > i32::MAX as i64 { return Err(ParseError::IntOverflow(s.clone())) }
            let out=v as i32; self.bump()?; Ok(out)
        } else { Err(ParseError::Unexpected{found:self.cur.clone(), expected:"un entier i32"}) }
    }
    fn parse_expr(&mut self)->Result<Expr,ParseError>{
        match &self.cur {
            Token::Str(s)=>{ let v=s.clone(); self.bump()?; Ok(Expr::Str(v)) }
            Token::Ident(s)=>{ let v=s.clone(); self.bump()?; Ok(Expr::Var(v)) }
            Token::Number(n)=>{ let v=n.parse::<i32>().map_err(|_|ParseError::IntOverflow(n.clone()))?; self.bump()?; Ok(Expr::Int(v)) }
            _ => Err(ParseError::Unexpected{found:self.cur.clone(), expected:"une expression (string | ident | int)"}),
        }
    }
    fn parse_log(&mut self)->Result<Stmt,ParseError>{
        self.expect(Token::Log,"`log`")?;
        self.expect(Token::LParen,"`(`")?;
        // au moins 1 arg
        let mut args=vec![ self.parse_expr()? ];
        while matches!(self.cur, Token::Comma) { self.bump()?; args.push(self.parse_expr()?); }
        self.expect(Token::RParen,"`)`")?;
        Ok(Stmt::Log(args))
    }
    fn parse_for(&mut self)->Result<Stmt,ParseError>{
        self.expect(Token::For,"`for`")?;
        let name = if let Token::Ident(s)=&self.cur { let v=s.clone(); self.bump()?; v }
                   else { return Err(ParseError::Unexpected{found:self.cur.clone(), expected:"identifiant"}) };
        self.expect(Token::Assign,"`=`")?;
        let start = self.parse_number_i32()?;
        self.expect(Token::To,"`to`")?;
        let end = self.parse_number_i32()?;
        self.expect(Token::LBrace,"`{`")?;
        let mut body=Vec::new();
        while !matches!(self.cur, Token::RBrace){ body.push(self.parse_stmt()?); }
        self.expect(Token::RBrace,"`}`")?;
        Ok(Stmt::For{ name, start, end, body })
    }
    fn parse_stmt(&mut self)->Result<Stmt,ParseError>{
        match self.cur {
            Token::Log => self.parse_log(),
            Token::For => self.parse_for(),
            _ => Err(ParseError::Unexpected{ found:self.cur.clone(), expected:"`log` ou `for`" }),
        }
    }
    pub fn parse_program(&mut self)->Result<Program,ParseError>{
        self.expect(Token::Fn,"`fn`")?;
        self.expect(Token::Main,"`main`")?;
        self.expect(Token::LParen,"`(`")?;
        self.expect(Token::RParen,"`)`")?;
        self.expect(Token::LBrace,"`{`")?;
        let mut stmts=Vec::new();
        while !matches!(self.cur, Token::RBrace){ stmts.push(self.parse_stmt()?); }
        self.expect(Token::RBrace,"`}`")?;
        self.expect(Token::Eof,"fin de fichier")?;
        Ok(Program{ stmts })
    }
}
