use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Constant(f64),
    Variable,
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
    Function(Function, Box<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Function { Sin, Cos, Tan, Sqrt, Exp, Ln, Abs }

impl Function {
    fn apply(self, x: f64) -> f64 {
        match self {
            Self::Sin => x.sin(), Self::Cos => x.cos(), Self::Tan => x.tan(),
            Self::Sqrt => x.sqrt(), Self::Exp => x.exp(), Self::Ln => x.ln(), Self::Abs => x.abs(),
        }
    }
}

impl Expr {
    pub fn eval(&self, x: f64) -> f64 {
        match self {
            Self::Constant(v) => *v,
            Self::Variable => x,
            Self::Add(a,b) => a.eval(x)+b.eval(x),
            Self::Sub(a,b) => a.eval(x)-b.eval(x),
            Self::Mul(a,b) => a.eval(x)*b.eval(x),
            Self::Div(a,b) => a.eval(x)/b.eval(x),
            Self::Pow(a,b) => a.eval(x).powf(b.eval(x)),
            Self::Neg(a) => -a.eval(x),
            Self::Function(f,a) => f.apply(a.eval(x)),
        }
    }

    pub fn sample(&self, min: f64, max: f64, count: usize) -> Vec<f64> {
        if count < 2 { return vec![self.eval(min)]; }
        let step = (max-min)/(count-1) as f64;
        (0..count).map(|i| self.eval(min+i as f64*step)).collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError { pub message: String, pub position: usize }
impl fmt::Display for ParseError {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result { write!(f,"{} at {}",self.message,self.position) }
}
impl std::error::Error for ParseError {}

#[derive(Debug, Clone, PartialEq)]
enum Tok { Num(f64), Ident(String), Plus, Minus, Star, Slash, Caret, LParen, RParen, End }

struct Lexer<'a>{src:&'a[u8],pos:usize}
impl<'a> Lexer<'a>{
 fn new(s:&'a str)->Self{Self{src:s.as_bytes(),pos:0}}
 fn lex(mut self)->Result<Vec<(Tok,usize)>,ParseError>{
  let mut out=Vec::new();
  while self.pos<self.src.len(){let p=self.pos;let c=self.src[self.pos]as char;match c{
   c if c.is_ascii_whitespace()=>self.pos+=1,
   '+'|'-'|'*'|'/'|'^'|'('|')'=>{self.pos+=1;out.push((match c{'+' =>Tok::Plus,'-' =>Tok::Minus,'*'=>Tok::Star,'/'=>Tok::Slash,'^'=>Tok::Caret,'('=>Tok::LParen,')'=>Tok::RParen,_=>unreachable!()},p));}
   c if c.is_ascii_digit()||c=='.'=>{let start=self.pos;while self.pos<self.src.len(){let ch=self.src[self.pos]as char;if !(ch.is_ascii_digit()||matches!(ch,'.'|'e'|'E'|'+'|'-')){break;}if matches!(ch,'+'|'-')&&self.pos>start&&!matches!(self.src[self.pos-1]as char,'e'|'E'){break;}self.pos+=1;}let raw=std::str::from_utf8(&self.src[start..self.pos]).unwrap();let n=raw.parse().map_err(|_|ParseError{message:"invalid number".into(),position:start})?;out.push((Tok::Num(n),start));}
   c if c.is_ascii_alphabetic()||c=='_'=>{let start=self.pos;while self.pos<self.src.len(){let ch=self.src[self.pos]as char;if !(ch.is_ascii_alphanumeric()||ch=='_'){break;}self.pos+=1;}out.push((Tok::Ident(std::str::from_utf8(&self.src[start..self.pos]).unwrap().into()),start));}
   _=>return Err(ParseError{message:format!("unexpected character '{c}'"),position:p})
  }}out.push((Tok::End,self.pos));Ok(out)
 }
}

struct Parser{toks:Vec<(Tok,usize)>,i:usize}
impl Parser{
 fn peek(&self)->&Tok{&self.toks[self.i].0}
 fn pos(&self)->usize{self.toks[self.i].1}
 fn bump(&mut self)->Tok{let t=self.toks[self.i].0.clone();self.i+=1;t}
 fn parse(mut self)->Result<Expr,ParseError>{let e=self.expr(0)?;if *self.peek()!=Tok::End{return Err(ParseError{message:"expected end of expression".into(),position:self.pos()})}Ok(e)}
 fn expr(&mut self,min_bp:u8)->Result<Expr,ParseError>{
  let mut lhs=self.prefix()?;
  loop{
   let(l,r,op)=match self.peek(){Tok::Plus=>(1,2,0),Tok::Minus=>(1,2,1),Tok::Star=>(3,4,2),Tok::Slash=>(3,4,3),Tok::Caret=>(6,5,4),Tok::RParen|Tok::End=>(0,0,255),Tok::Ident(_)|Tok::Num(_)|Tok::LParen=>(3,4,2)};
   if l<min_bp||op==255{break}
   let explicit=matches!(self.peek(),Tok::Plus|Tok::Minus|Tok::Star|Tok::Slash|Tok::Caret);
   if !explicit{let rhs=self.expr(r)?;lhs=Expr::Mul(Box::new(lhs),Box::new(rhs));continue}
   self.bump();let rhs=self.expr(r)?;
   lhs=match op{0=>Expr::Add(Box::new(lhs),Box::new(rhs)),1=>Expr::Sub(Box::new(lhs),Box::new(rhs)),2=>Expr::Mul(Box::new(lhs),Box::new(rhs)),3=>Expr::Div(Box::new(lhs),Box::new(rhs)),4=>Expr::Pow(Box::new(lhs),Box::new(rhs)),_=>unreachable!()};
  }
  Ok(lhs)
 }
 fn prefix(&mut self)->Result<Expr,ParseError>{
  match self.bump(){
   Tok::Num(n)=>Ok(Expr::Constant(n)),
   Tok::Ident(name)=>match name.as_str(){
    "x"=>Ok(Expr::Variable),
    "pi"=>Ok(Expr::Constant(std::f64::consts::PI)),
    "e"=>Ok(Expr::Constant(std::f64::consts::E)),
    _=>{let f=match name.as_str(){"sin"=>Function::Sin,"cos"=>Function::Cos,"tan"=>Function::Tan,"sqrt"=>Function::Sqrt,"exp"=>Function::Exp,"ln"|"log"=>Function::Ln,"abs"=>Function::Abs,_=>return Err(ParseError{message:format!("unknown identifier '{name}'"),position:self.pos().saturating_sub(name.len())})};if *self.peek()!=Tok::LParen{return Err(ParseError{message:"expected '(' after function".into(),position:self.pos()})}self.bump();let arg=self.expr(0)?;if *self.peek()!=Tok::RParen{return Err(ParseError{message:"expected ')'".into(),position:self.pos()})}self.bump();Ok(Expr::Function(f,Box::new(arg)))}
   },
   Tok::Minus=>Ok(Expr::Neg(Box::new(self.expr(5)?))),
   Tok::Plus=>self.prefix(),
   Tok::LParen=>{let e=self.expr(0)?;if *self.peek()!=Tok::RParen{return Err(ParseError{message:"expected ')'".into(),position:self.pos()})}self.bump();Ok(e)},
   t=>Err(ParseError{message:format!("unexpected token {t:?}"),position:self.pos()})
  }
 }
}

pub fn parse(source:&str)->Result<Expr,ParseError>{Parser{toks:Lexer::new(source).lex()?,i:0}.parse()}

#[cfg(test)]
mod tests {
 use super::*;
 #[test] fn expressions(){let e=parse("sin(x) + 0.5cos(3x)").unwrap();assert!((e.eval(0.0)-0.5).abs()<1e-10);assert_eq!(parse("2^3^2").unwrap().eval(0.0),512.0);}
}
