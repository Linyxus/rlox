use crate::compiler::Compiler;
use crate::span::Span;

#[derive(Debug)]
#[derive(PartialEq, Eq, Hash)]
#[derive(Clone, Copy)]
pub enum TokenType {
    LeftParen, RightParen,
    LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus,
    SemiColon, Slash, Star,

    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    Identifier, String, Number,

    And, Class, Else, False,
    For, Fun, If, Nil, Or,
    Print, Return, Super, This,
    True, Var, While,

    Error, EOF
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Token {
    pub tp: TokenType,
    pub span: Span,
    pub content: String,
    pub line: u32,
}

impl Token {
    pub fn show(&self) -> String {
        match self.tp {
            TokenType::Error => format!("<error: {}>", self.content),
            TokenType::EOF => "<eof>".into(),
            TokenType::Identifier => format!("@{}", self.content),
            _ => self.content.clone().into(),
        }
    }
}

#[derive(Debug)]
pub struct ScannerState {
    pub start: u32,
    pub current: u32,
    pub line: u32
}

impl ScannerState {
    pub fn new() -> ScannerState {
        ScannerState { start: 0, current: 0, line: 1 }
    }
}

fn is_eof(compiler: &Compiler) -> bool {
    compiler.scanner.current as usize >= compiler.source.len()
}

fn make_token(compiler: &Compiler, tp: TokenType) -> Token {
    let start = compiler.scanner.start as usize;
    let end = compiler.scanner.current as usize;
    Token {
        tp,
        span: Span::new(start, end - start),
        content: compiler.source[start..end].into(),
        line: compiler.scanner.line
    }
}

fn error_token(compiler: &Compiler, msg: String) -> Token {
    Token {
        tp: TokenType::Error,
        span: Span::new(0, 0),
        content: msg,
        line: compiler.scanner.line
    }
}

fn peek(compiler: &Compiler) -> char {
    let pos = compiler.scanner.current;
    compiler.source.as_bytes()[pos as usize] as char
}

fn peek_next(compiler: &Compiler) -> Option<char> {
    let pos = compiler.scanner.current + 1;
    if (pos as usize) < compiler.source.len() {
        Some(compiler.source.as_bytes()[pos as usize] as char)
    } else {
        None
    }
}

fn advance(compiler: &mut Compiler) -> char {
    if is_eof(compiler) {
        panic!("EOF!")
    } else {
        compiler.scanner.current += 1;
        compiler.source.as_bytes()[(compiler.scanner.current - 1) as usize] as char
    }
}

pub fn next_token(compiler: &mut Compiler) -> Token {
    skip_whitespaces(compiler);

    compiler.scanner.start = compiler.scanner.current;

    if is_eof(compiler) {
        return make_token(compiler, TokenType::EOF)
    }

    let c = advance(compiler);

    match c {
        '(' => return make_token(compiler, TokenType::LeftParen),
        ')' => return make_token(compiler, TokenType::RightParen),
        '{' => return make_token(compiler, TokenType::LeftBrace),
        '}' => return make_token(compiler, TokenType::RightBrace),
        ';' => return make_token(compiler, TokenType::SemiColon),
        ',' => return make_token(compiler, TokenType::Comma),
        '.' => return make_token(compiler, TokenType::Dot),
        '-' => return make_token(compiler, TokenType::Minus),
        '+' => return make_token(compiler, TokenType::Plus),
        '/' => return make_token(compiler, TokenType::Slash),
        '*' => return make_token(compiler, TokenType::Star),
        '!' => {
            let tp = if match_ahead(compiler, '=') { TokenType::BangEqual } else { TokenType::Bang };
            return make_token(compiler, tp)
        },
        '=' => {
            let tp = if match_ahead(compiler, '=') { TokenType::EqualEqual } else { TokenType::Equal };
            return make_token(compiler, tp)
        },
        '>' => {
            let tp = if match_ahead(compiler, '=') { TokenType::GreaterEqual } else { TokenType::Greater };
            return make_token(compiler, tp)
        },
        '<' => {
            let tp = if match_ahead(compiler, '=') { TokenType::LessEqual } else { TokenType::Less };
            return make_token(compiler, tp)
        },
        '"' => return scan_string(compiler),
        ch if is_digit(ch) => return scan_number(compiler),
        ch if is_alpha_underscore(ch) => return scan_identifier(compiler),
        _ => (),
    }

    error_token(compiler, format!("Fail to tokenize at character '{}'", c))
}

fn skip_whitespaces(compiler: &mut Compiler) {
    loop {
        if is_eof(compiler) { break; }

        let ch = peek(compiler);
        match ch {
            ' ' | '\r' | '\t' => { advance(compiler); },
            '\n' => { advance(compiler); compiler.scanner.line += 1; },
            '/' => {
                match peek_next(compiler) {
                    Some('/') => {
                        while !is_eof(compiler) && peek(compiler) != '\n' {
                            advance(compiler);
                        }
                    },
                    _ => { break; },
                }
            }
            _ => { break; },
        }
    }
}

fn match_ahead(compiler: &mut Compiler, expected: char) -> bool {
    if is_eof(compiler) { false } else {
        let ch = peek(compiler);
        if ch == expected { advance(compiler); }
        ch == expected
    }
}

fn scan_string(compiler: &mut Compiler) -> Token {
    while !is_eof(compiler) && peek(compiler) != '"' {
        if peek(compiler) == '\n' { compiler.scanner.line += 1; }
        advance(compiler);
    }

    if is_eof(compiler) {
        error_token(compiler, "Non-terminated string literal".into())
    } else {
        advance(compiler);
        make_token(compiler, TokenType::String)
    }
}

fn is_digit(ch: char) -> bool {
    ch >= '0' && ch <= '9'
}

fn scan_number(compiler: &mut Compiler) -> Token {
    while !is_eof(compiler) && is_digit(peek(compiler)) {
        advance(compiler);
    }

    if peek(compiler) == '.' {  // start scaning fraction part
        advance(compiler);

        while !is_eof(compiler) && is_digit(peek(compiler)) {
            advance(compiler);
        }
    }

    make_token(compiler, TokenType::Number)
}

fn is_alpha(ch: char) -> bool {
    (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z')
}

fn is_alpha_underscore(ch: char) -> bool {
    is_alpha(ch) || ch == '_'
}

fn keyword_or_identifier(content: &str) -> TokenType {
    match content {
        "and" => TokenType::And,
        "class" => TokenType::Class,
        "else" => TokenType::Else,
        "false" => TokenType::False,
        "for" => TokenType::For,
        "fun" => TokenType::Fun,
        "if" => TokenType::If,
        "nil" => TokenType::Nil,
        "or" => TokenType::Or,
        "print" => TokenType::Print,
        "return" => TokenType::Return,
        "super" => TokenType::Super,
        "this" => TokenType::This,
        "true" => TokenType::True,
        "var" => TokenType::Var,
        "while" => TokenType::While,
        _ => TokenType::Identifier,
    }
}

fn scan_identifier(compiler: &mut Compiler) -> Token {
    while !is_eof(compiler) && is_alpha_underscore(peek(compiler)) {
        advance(compiler);
    }

    let start = compiler.scanner.start as usize;
    let current = compiler.scanner.current as usize;

    make_token(compiler, keyword_or_identifier(&compiler.source[start..current]))
}

