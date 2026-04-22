use std::fmt;
use crate::lexer::Token;

// ============================================================================
// AST TYPES
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Param {
    Name(String),
    Destructure(Vec<String>),  // {name, age}
    Rest(String),              // ...args
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Str(String),
    Ident(String),
    Binary(Box<Expr>, BinOp, Box<Expr>),
    Conditional(Box<Expr>, Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Function(Vec<Param>, Vec<Stmt>),
    ArrayLiteral(Vec<Expr>),
    ObjectLiteral(Vec<(String, Expr)>),
    Index(Box<Expr>, Box<Expr>),
    PropAccess(Box<Expr>, String),
    MethodCall(Box<Expr>, String, Vec<Expr>),
    Spread(Box<Expr>),
    New(Box<Expr>, Vec<Expr>),  // new ctor(args)
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Gt, Le, Ge,
    And, Or,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Mod => write!(f, "%"),
            BinOp::Eq => write!(f, "=="),
            BinOp::Ne => write!(f, "!="),
            BinOp::Lt => write!(f, "<"),
            BinOp::Gt => write!(f, ">"),
            BinOp::Le => write!(f, "<="),
            BinOp::Ge => write!(f, ">="),
            BinOp::And => write!(f, "&&"),
            BinOp::Or => write!(f, "||"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    FuncDecl(FuncDecl),
    Return(Expr),
    Expr(Expr),  // Expression statement (no return)
    Import(ImportStmt),
    Export(ExportStmt),
}

/// `import { name1, name2 } from "module_path";`
#[derive(Debug, Clone, PartialEq)]
pub struct ImportStmt {
    pub names: Vec<String>,        // destructured names
    pub source: String,             // module path string
}

/// `export function name(...) { ... }`
#[derive(Debug, Clone, PartialEq)]
pub struct ExportStmt {
    pub func: FuncDecl,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuncDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
}

// ============================================================================
// PARSER
// ============================================================================

pub struct Parser {
    pub tokens: Vec<Token>,
    pub pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::EOF)
    }

    pub fn consume(&mut self) -> Token {
        let token = self.tokens.get(self.pos).unwrap_or(&Token::EOF).clone();
        self.pos += 1;
        token
    }

    pub fn expect(&mut self, expected: &str) -> Result<(), String> {
        let token = self.consume();
        let matches = match (&token, expected) {
            (Token::LParen, "LParen") => true,
            (Token::RParen, "RParen") => true,
            (Token::LBrace, "LBrace") => true,
            (Token::RBrace, "RBrace") => true,
            (Token::LBracket, "LBracket") => true,
            (Token::RBracket, "RBracket") => true,
            (Token::Comma, "Comma") => true,
            (Token::Semicolon, "Semicolon") => true,
            (Token::Colon, "Colon") => true,
            (Token::Dot, "Dot") => true,
            _ => false,
        };
        if !matches {
            Err(format!("Expected {}, got {:?}", expected, token))
        } else {
            Ok(())
        }
    }

    pub fn parse_func_decl(&mut self) -> Result<FuncDecl, String> {
        self.consume();
        let name = match self.consume() {
            Token::Ident(n) => n,
            t => return Err(format!("Expected function name, got {:?}", t)),
        };
        self.expect("LParen")?;
        let params = self.parse_params()?;
        self.expect("RParen")?;
        self.expect("LBrace")?;
        let body = self.parse_stmts()?;
        self.expect("RBrace")?;
        Ok(FuncDecl { name, params, body })
    }

    pub fn parse_params(&mut self) -> Result<Vec<Param>, String> {
        let mut params = Vec::new();
        if matches!(self.peek(), Token::RParen) {
            return Ok(params);
        }
        loop {
            // Rest param: ...args
            if matches!(self.peek(), Token::Spread) {
                self.consume();
                match self.consume() {
                    Token::Ident(name) => {
                        params.push(Param::Rest(name));
                        return Ok(params);
                    }
                    t => return Err(format!("Expected param name after ..., got {:?}", t)),
                }
            } else if matches!(self.peek(), Token::LBrace) {
                // Destructuring param: {name, age}
                self.consume();
                let mut fields = Vec::new();
                loop {
                    match self.consume() {
                        Token::Ident(f) => fields.push(f),
                        t => return Err(format!("Expected field name in destructuring, got {:?}", t)),
                    }
                    if matches!(self.peek(), Token::Comma) {
                        self.consume();
                    } else {
                        break;
                    }
                }
                self.expect("RBrace")?;
                params.push(Param::Destructure(fields));
            } else {
                match self.consume() {
                    Token::Ident(p) => params.push(Param::Name(p)),
                    t => return Err(format!("Expected param name, got {:?}", t)),
                }
            }
            if matches!(self.peek(), Token::Comma) {
                self.consume();
            } else {
                break;
            }
        }
        Ok(params)
    }

    pub fn parse_stmts(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        // import { a, b } from "path";
        if matches!(self.peek(), Token::Import) {
            self.consume();
            let import = self.parse_import()?;
            stmts.push(Stmt::Import(import));
        }
        while matches!(self.peek(), Token::Function) {
            self.consume();
            if matches!(self.peek(), Token::Ident(_)) {
                let name = match self.consume() {
                    Token::Ident(n) => n,
                    t => return Err(format!("Expected function name, got {:?}", t)),
                };
                self.expect("LParen")?;
                let params = self.parse_params()?;
                self.expect("RParen")?;
                self.expect("LBrace")?;
                let body = self.parse_stmts()?;
                self.expect("RBrace")?;
                stmts.push(Stmt::FuncDecl(FuncDecl { name, params, body }));
            } else {
                self.expect("LParen")?;
                let params = self.parse_params()?;
                self.expect("RParen")?;
                self.expect("LBrace")?;
                let body = self.parse_stmts()?;
                self.expect("RBrace")?;
                stmts.push(Stmt::Return(Expr::Function(params, body)));
            }
        }
        // export function name(...) { ... }
        if matches!(self.peek(), Token::Export) {
            self.consume();
            let export = self.parse_export()?;
            stmts.push(Stmt::Export(export));
        }
        // Parse remaining statements (return or expression)
        loop {
            if matches!(self.peek(), Token::Return) {
                self.consume();
                let expr = self.parse_conditional()?;
                if matches!(self.peek(), Token::Semicolon) {
                    self.consume();
                }
                stmts.push(Stmt::Return(expr));
            } else if !matches!(self.peek(), Token::RBrace | Token::EOF) {
                // Bare expression statement - check if there's more content after
                let save_pos = self.pos;
                let expr = self.parse_conditional()?;
                if matches!(self.peek(), Token::Semicolon) {
                    self.consume();
                }
                // Look ahead to see if there's more content (another statement)
                if matches!(self.peek(), Token::RBrace | Token::EOF) {
                    // This is the last statement - treat as return
                    stmts.push(Stmt::Return(expr));
                } else {
                    // There's more content - treat as expression statement
                    stmts.push(Stmt::Expr(expr));
                }
            } else {
                break;
            }
        }
        Ok(stmts)
    }

    pub fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_conditional()
    }

    pub fn parse_conditional(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_comparison()?;
        if matches!(self.peek(), Token::Question) {
            self.consume();
            let then_expr = self.parse_conditional()?;
            self.expect("Colon")?;
            let else_expr = self.parse_conditional()?;
            expr = Expr::Conditional(Box::new(expr), Box::new(then_expr), Box::new(else_expr));
        }
        Ok(expr)
    }

    pub fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.peek() {
                Token::Eq => BinOp::Eq,
                Token::Ne => BinOp::Ne,
                Token::Lt => BinOp::Lt,
                Token::Gt => BinOp::Gt,
                Token::Le => BinOp::Le,
                Token::Ge => BinOp::Ge,
                Token::And => BinOp::And,
                Token::Or => BinOp::Or,
                _ => break,
            };
            self.consume();
            let right = self.parse_additive()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    pub fn parse_additive(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.peek() {
                Token::Add => BinOp::Add,
                Token::Sub => BinOp::Sub,
                _ => break,
            };
            self.consume();
            let right = self.parse_multiplicative()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    pub fn parse_multiplicative(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_chain()?;
        loop {
            let op = match self.peek() {
                Token::Mul => BinOp::Mul,
                Token::Div => BinOp::Div,
                Token::Mod => BinOp::Mod,
                _ => break,
            };
            self.consume();
            let right = self.parse_chain()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    pub fn parse_chain(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek() {
                Token::LBracket => {
                    self.consume();
                    let index = self.parse_conditional()?;
                    self.expect("RBracket")?;
                    expr = Expr::Index(Box::new(expr), Box::new(index));
                }
                Token::LParen => {
                    self.consume();
                    let args = self.parse_call_args()?;
                    self.expect("RParen")?;
                    expr = Expr::Call(Box::new(expr), args);
                }
                Token::Dot => {
                    self.consume();
                    let prop = match self.consume() {
                        Token::Ident(n) => n,
                        t => return Err(format!("Expected property name, got {:?}", t)),
                    };
                    if matches!(self.peek(), Token::LParen) {
                        self.consume();
                        let args = self.parse_call_args()?;
                        self.expect("RParen")?;
                        expr = Expr::MethodCall(Box::new(expr), prop, args);
                    } else {
                        expr = Expr::PropAccess(Box::new(expr), prop);
                    }
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    pub fn parse_primary(&mut self) -> Result<Expr, String> {
        let expr = match self.peek() {
            Token::Number(n) => {
                let val = *n;
                self.consume();
                Ok(Expr::Number(val))
            }
            Token::StrLit(s) => {
                let val = s.clone();
                self.consume();
                Ok(Expr::Str(val))
            }
            Token::Ident(name) => {
                let name = name.clone();
                self.consume();
                Ok(Expr::Ident(name))
            }
            Token::Function => {
                self.consume();
                if matches!(self.peek(), Token::Ident(_)) {
                    let _name = match self.consume() {
                        Token::Ident(n) => n,
                        t => return Err(format!("Expected function name, got {:?}", t)),
                    };
                    self.expect("LParen")?;
                    let params = self.parse_params()?;
                    self.expect("RParen")?;
                    self.expect("LBrace")?;
                    let body = self.parse_stmts()?;
                    self.expect("RBrace")?;
                    Ok(Expr::Function(params, body))
                } else {
                    self.expect("LParen")?;
                    let params = self.parse_params()?;
                    self.expect("RParen")?;
                    self.expect("LBrace")?;
                    let body = self.parse_stmts()?;
                    self.expect("RBrace")?;
                    Ok(Expr::Function(params, body))
                }
            }
            Token::New => {
                self.consume();
                let ctor = self.parse_primary()?;
                self.expect("LParen")?;
                let args = self.parse_call_args()?;
                self.expect("RParen")?;
                Ok(Expr::New(Box::new(ctor), args))
            }
            Token::LBracket => {
                self.consume();
                let elements = if matches!(self.peek(), Token::RBracket) {
                    Vec::new()
                } else {
                    let mut elements = Vec::new();
                    loop {
                        if matches!(self.peek(), Token::Spread) {
                            self.consume();
                            let expr = self.parse_conditional()?;
                            elements.push(Expr::Spread(Box::new(expr)));
                        } else {
                            elements.push(self.parse_conditional()?);
                        }
                        if matches!(self.peek(), Token::Comma) {
                            self.consume();
                        } else {
                            break;
                        }
                    }
                    elements
                };
                self.expect("RBracket")?;
                Ok(Expr::ArrayLiteral(elements))
            }
            Token::LBrace => {
                self.consume();
                let mut entries = Vec::new();
                if !matches!(self.peek(), Token::RBrace) {
                    loop {
                        let key = match self.consume() {
                            Token::Ident(k) => k,
                            Token::StrLit(k) => k,
                            t => return Err(format!("Expected object key, got {:?}", t)),
                        };
                        if matches!(self.peek(), Token::Colon) {
                            self.consume();
                            let value = self.parse_conditional()?;
                            entries.push((key, value));
                        } else {
                            entries.push((key.clone(), Expr::Ident(key)));
                        }
                        if matches!(self.peek(), Token::Comma) {
                            self.consume();
                        } else {
                            break;
                        }
                    }
                }
                self.expect("RBrace")?;
                Ok(Expr::ObjectLiteral(entries))
            }
            Token::LParen => {
                self.consume();
                let expr = self.parse_conditional()?;
                self.expect("RParen")?;
                Ok(expr)
            }
            _ => Err(format!("Unexpected token: {:?}", self.peek())),
        }?;
        Ok(expr)
    }

    pub fn parse_call_args(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        if matches!(self.peek(), Token::RParen) {
            return Ok(args);
        }
        loop {
            if matches!(self.peek(), Token::Spread) {
                self.consume();
                let expr = self.parse_conditional()?;
                args.push(Expr::Spread(Box::new(expr)));
            } else {
                args.push(self.parse_conditional()?);
            }
            if matches!(self.peek(), Token::Comma) {
                self.consume();
            } else {
                break;
            }
        }
        Ok(args)
    }

    /// Parse: import { name1, name2 } from "module_path";
    pub fn parse_import(&mut self) -> Result<ImportStmt, String> {
        // expect {
        self.expect("LBrace")?;
        let mut names = Vec::new();
        loop {
            match self.consume() {
                Token::Ident(n) => names.push(n),
                t => return Err(format!("Expected import name, got {:?}", t)),
            }
            if matches!(self.peek(), Token::Comma) {
                self.consume();
            } else {
                break;
            }
        }
        self.expect("RBrace")?;
        // expect from
        if !matches!(self.peek(), Token::From) {
            return Err("Expected 'from' after import destructuring".into());
        }
        self.consume();
        // expect string
        match self.consume() {
            Token::StrLit(path) => Ok(ImportStmt { names, source: path }),
            t => Err(format!("Expected module path, got {:?}", t)),
        }
    }

    /// Parse: export function name(...) { ... }
    pub fn parse_export(&mut self) -> Result<ExportStmt, String> {
        // expect function
        if !matches!(self.peek(), Token::Function) {
            return Err("Expected 'function' after export".into());
        }
        self.consume();
        let name = match self.consume() {
            Token::Ident(n) => n,
            t => return Err(format!("Expected function name after export, got {:?}", t)),
        };
        self.expect("LParen")?;
        let params = self.parse_params()?;
        self.expect("RParen")?;
        self.expect("LBrace")?;
        let body = self.parse_stmts()?;
        self.expect("RBrace")?;
        Ok(ExportStmt { func: FuncDecl { name, params, body } })
    }
}
