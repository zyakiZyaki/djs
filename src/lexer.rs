// ============================================================================
// TOKENS
// ============================================================================

#[derive(Debug, Clone)]
pub enum Token {
    Function, New, Return, Import, Export, From,
    Ident(String),
    Number(f64),
    StrLit(String),
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Comma, Semicolon, Dot,
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Gt, Le, Ge,
    And, Or, Question, Colon,
    Spread,  // ...
    EOF,
}

// ============================================================================
// LEXER
// ============================================================================

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> char {
        if self.pos < self.input.len() {
            self.input[self.pos]
        } else {
            '\0'
        }
    }

    fn next(&mut self) -> char {
        let ch = self.peek();
        self.pos += 1;
        ch
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            if self.peek().is_whitespace() {
                self.next();
            } else if self.peek() == '/' && self.pos + 1 < self.input.len() && self.input[self.pos + 1] == '/' {
                while self.pos < self.input.len() && self.peek() != '\n' {
                    self.next();
                }
            } else if self.peek() == '/' && self.pos + 1 < self.input.len() && self.input[self.pos + 1] == '*' {
                self.next(); self.next();
                while self.pos + 1 < self.input.len() {
                    if self.peek() == '*' && self.input[self.pos + 1] == '/' {
                        self.next(); self.next();
                        break;
                    }
                    self.next();
                }
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> f64 {
        let mut num = String::new();
        while self.pos < self.input.len() && (self.peek().is_digit(10) || self.peek() == '.') {
            num.push(self.next());
        }
        num.parse().unwrap_or(0.0)
    }

    fn read_ident(&mut self) -> String {
        let mut ident = String::new();
        while self.pos < self.input.len() && (self.peek().is_alphanumeric() || self.peek() == '_') {
            ident.push(self.next());
        }
        ident
    }

    fn read_string(&mut self) -> String {
        let mut s = String::new();
        let quote = self.next(); // consume opening quote
        while self.pos < self.input.len() {
            if self.peek() == '\\' && self.pos + 1 < self.input.len() {
                self.next();
                match self.next() {
                    '"' => s.push('"'),
                    '\\' => s.push('\\'),
                    'n' => s.push('\n'),
                    't' => s.push('\t'),
                    'r' => s.push('\r'),
                    other => { s.push('\\'); s.push(other); }
                }
            } else if self.peek() == quote {
                self.next();
                break;
            } else {
                s.push(self.next());
            }
        }
        s
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        if self.pos >= self.input.len() {
            return Token::EOF;
        }
        let ch = self.peek();
        if ch.is_digit(10) {
            return Token::Number(self.read_number());
        }
        if ch == '"' || ch == '\'' {
            return Token::StrLit(self.read_string());
        }
        if ch.is_alphabetic() || ch == '_' {
            let ident = self.read_ident();
            return match ident.as_str() {
                "new" => Token::New,
                "function" => Token::Function,
                "return" => Token::Return,
                "import" => Token::Import,
                "export" => Token::Export,
                "from" => Token::From,
                "true" => Token::Number(1.0),
                "false" => Token::Number(0.0),
                _ => Token::Ident(ident),
            };
        }
        self.next();
        match ch {
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            ',' => Token::Comma,
            ';' => Token::Semicolon,
            '.' => {
                // Check for spread: ...
                if self.pos < self.input.len() && self.input[self.pos] == '.' {
                    self.next();
                    if self.pos < self.input.len() && self.input[self.pos] == '.' {
                        self.next();
                        Token::Spread
                    } else {
                        Token::Dot
                    }
                } else {
                    Token::Dot
                }
            }
            '+' => Token::Add,
            '-' => Token::Sub,
            '*' => Token::Mul,
            '/' => Token::Div,
            '%' => Token::Mod,
            '?' => Token::Question,
            ':' => Token::Colon,
            '=' if self.pos < self.input.len() && self.input[self.pos] == '=' => { self.next(); Token::Eq }
            '!' if self.pos < self.input.len() && self.input[self.pos] == '=' => { self.next(); Token::Ne }
            '<' if self.pos < self.input.len() && self.input[self.pos] == '=' => { self.next(); Token::Le }
            '>' if self.pos < self.input.len() && self.input[self.pos] == '=' => { self.next(); Token::Ge }
            '&' if self.pos < self.input.len() && self.input[self.pos] == '&' => { self.next(); Token::And }
            '|' if self.pos < self.input.len() && self.input[self.pos] == '|' => { self.next(); Token::Or }
            '<' => Token::Lt,
            '>' => Token::Gt,
            '!' => Token::Ne,
            _ => self.next_token(),
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if matches!(token, Token::EOF) {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        tokens
    }
}
