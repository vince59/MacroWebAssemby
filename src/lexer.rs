#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Fn, Main, Log, For, To,
    Ident(String),
    Number(String),     // i32 décimal seulement
    LParen, RParen, LBrace, RBrace,
    Comma, Assign,      // , =
    Str(String),
    Eof,
}

#[derive(Debug, Clone)]
pub struct LexError {
    pub message: String,
    pub at_byte: usize,
}
impl std::fmt::Display for LexError {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
        write!(f,"{} (byte {})",self.message,self.at_byte)
    }
}
impl std::error::Error for LexError {}

pub struct Lexer<'a> {
    input: &'a str,
    b: &'a [u8],
    i: usize,
}
impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self { Self { input, b: input.as_bytes(), i:0 } }
    fn eof(&self)->bool { self.i>=self.b.len() }
    fn peek(&self)->Option<u8>{ self.b.get(self.i).copied() }
    fn bump(&mut self)->Option<u8>{ let x=self.peek()?; self.i+=1; Some(x) }
    fn skip_ws(&mut self){ while let Some(c)=self.peek(){ match c{ b' '|b'\t'|b'\r'|b'\n'=>self.i+=1,_=>break } } }

    fn is_ident_start(b:u8)->bool{ (b'a'..=b'z').contains(&b) || (b'A'..=b'Z').contains(&b) || b==b'_' }
    fn is_ident_continue(b:u8)->bool{ Self::is_ident_start(b) || (b'0'..=b'9').contains(&b) }

    fn read_ident(&mut self)->(&'a str,usize,usize){
        let s=self.i;
        while let Some(c)=self.peek(){ if Self::is_ident_continue(c){ self.i+=1 } else { break } }
        (&self.input[s..self.i], s, self.i)
    }

    fn read_number(&mut self)->(&'a str,usize,usize){
        let s=self.i;
        while let Some(c)=self.peek(){ if (b'0'..=b'9').contains(&c){ self.i+=1 } else { break } }
        (&self.input[s..self.i], s, self.i)
    }

    fn read_string(&mut self)->Result<Token,LexError>{
        let start=self.i;
        self.bump(); // "
        let s=self.i;
        while let Some(c)=self.peek(){
            if c==b'"' { let out=&self.input[s..self.i]; self.i+=1; return Ok(Token::Str(out.to_string())) }
            self.i+=1;
        }
        Err(LexError{message:"chaine non terminée".into(), at_byte:start})
    }

    pub fn next_token(&mut self)->Result<Token,LexError>{
        self.skip_ws();
        if self.eof(){ return Ok(Token::Eof) }
        match self.peek().unwrap(){
            b'('=>{self.i+=1; Ok(Token::LParen)}
            b')'=>{self.i+=1; Ok(Token::RParen)}
            b'{' =>{self.i+=1; Ok(Token::LBrace)}
            b'}' =>{self.i+=1; Ok(Token::RBrace)}
            b',' =>{self.i+=1; Ok(Token::Comma)}
            b'=' =>{self.i+=1; Ok(Token::Assign)}
            b'"' => self.read_string(),
            c if Self::is_ident_start(c) =>{
                let (id,at,_) = self.read_ident();
                Ok(match id {
                    "fn"=>Token::Fn, "main"=>Token::Main, "log"=>Token::Log,
                    "for"=>Token::For, "to"=>Token::To,
                    _ => Token::Ident(id.to_string()),
                })
            }
            c if (b'0'..=b'9').contains(&c) => {
                let (n,_,_) = self.read_number();
                Ok(Token::Number(n.to_string()))
            }
            other => Err(LexError{message:format!("caractère 0x{other:02X} inattendu"), at_byte:self.i}),
        }
    }
}
