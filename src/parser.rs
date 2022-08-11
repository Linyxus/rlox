use crate::chunk::{Inst, KMethod};
use crate::scanner::{Token, TokenType};
use crate::compiler::Compiler;
use crate::span::Span;
use crate::value::Value;
use crate::obj::Obj;

use std::collections::HashMap;
use std::rc::Rc;

pub struct ParserState {
    pub current: Token,
    pub previous: Token,
    pub had_error: bool,
    pub panic_mode: bool,
    pub parsing_table: ParseTable,
}

impl ParserState {
    pub fn new() -> ParserState {
        ParserState {
            current: empty_token(),
            previous: empty_token(),
            had_error: false,
            panic_mode: false,
            parsing_table: ParseRule::make_rules()
        }
    }

    pub fn get_rule(&self, tp: TokenType) -> &ParseRule {
        &self.parsing_table[&tp]
    }
}

fn empty_token() -> Token {
    Token { tp: TokenType::Error, span: Span::new(0, 0), content: "EMPTY TOKEN".into(), line: 0 }
}

fn error_at(parser: &mut ParserState, token: &Token, msg: &str) {
    if parser.panic_mode {
        return;
    }
    parser.panic_mode = true;

    print!("[line {}] Error", token.line);

    match token.tp {
        TokenType::EOF => print!(" at End"),
        TokenType::Error => {},
        _ => print!(" at {}", token.content),
    }

    println!(" : {}", msg);

    parser.had_error = true;
}

fn emit_error(compiler: &mut Compiler, msg: &str) {
    let tok = compiler.parser.previous.clone();
    error_at(&mut compiler.parser, &tok, msg);
}

fn emit_error_at_current(compiler: &mut Compiler, msg: &str) {
    let tok = compiler.parser.current.clone();
    error_at(&mut compiler.parser, &tok, msg);
}

pub fn advance(compiler: &mut Compiler) {
    compiler.parser.previous = compiler.parser.current.clone();

    loop {
        let tok = compiler.next_token();

        if tok.tp != TokenType::Error {
            compiler.parser.current = tok;
            break;
        }

        emit_error_at_current(compiler, &tok.content);
    }
}

pub fn consume(compiler: &mut Compiler, tp: TokenType, msg: &str) {
    if compiler.parser.current.tp == tp {
        advance(compiler);
    } else {
        let current_token: Token = compiler.parser.current.clone();
        error_at(&mut compiler.parser, &current_token, &msg[..]);
    }
}

pub fn check_next(compiler: &Compiler, tp: TokenType) -> bool {
    compiler.parser.current.tp == tp
}

pub fn try_consume(compiler: &mut Compiler, tp: TokenType) -> bool {
    check_next(compiler, tp) && {
        consume(compiler, tp, "");
        true
    }
}

fn emit_str(compiler: &mut Compiler, s: String) {
    let r = Rc::new(Obj::Str { data: s });
    let v = Value::OBJ { data: r };

    emit_constant(compiler, v)
}

fn make_str(compiler: &mut Compiler, s: String) -> usize {
    let r = Rc::new(Obj::Str { data: s });
    let v = Value::OBJ { data: r };

    compiler.current_chunk.value_array.add_constant(v)
}

fn emit_constant(compiler: &mut Compiler, v: Value) {
    let idx = compiler.current_chunk.value_array.add_constant(v);
    compiler.emit_inst(Inst::CONSTANT { idx });
}

fn emit_print(compiler: &mut Compiler) {
    compiler.emit_inst(Inst::OP_KCALL { tp: KMethod::Print })
}

pub fn parse_expression(compiler: &mut Compiler) {
    parse_prec(compiler, Precedence::Assignment);
}

pub fn parse_decl(compiler: &mut Compiler) {
    if try_consume(compiler, TokenType::Var) {
        parse_var_decl(compiler);
    } else {
        parse_stmt(compiler);
    }
}

fn parse_var_decl(compiler: &mut Compiler) {
    let varname_idx = parse_var(compiler, "Expecting variable name after `var`");

    if try_consume(compiler, TokenType::Equal) {
        parse_expression(compiler);
    } else {
        emit_constant(compiler, Value::NIL);
    }

    consume(compiler, TokenType::SemiColon, "Expecting ';' after variable decl");

    define_variable(compiler, varname_idx);
}

fn define_variable(compiler: &mut Compiler, varname_idx: usize) {
    compiler.emit_inst(Inst::OP_DEFINE_GLOBAL { name_idx: varname_idx });
}

fn parse_var(compiler: &mut Compiler, err_msg: &str) -> usize {
    consume(compiler, TokenType::Identifier, err_msg);
    let var_name = compiler.parser.previous.content.clone();
    make_str(compiler, var_name)
}

pub fn parse_stmt(compiler: &mut Compiler) {
    if try_consume(compiler, TokenType::Print) {
        parse_print_stmt(compiler);
    } else {
        parse_expr_stmt(compiler);
    }
}

fn parse_expr_stmt(compiler: &mut Compiler) {
    parse_expression(compiler);
    consume(compiler, TokenType::SemiColon, "Expect ';' at end of statement.");
    compiler.emit_inst(Inst::OP_POP);
}

fn parse_print_stmt(compiler: &mut Compiler) {
    parse_expression(compiler);
    consume(compiler, TokenType::SemiColon, "Expect ';' at end of statement.");
    emit_print(compiler);
}

fn parse_prec(compiler: &mut Compiler, prec: Precedence) {
    advance(compiler);
    let prev = &compiler.parser.previous;
    let prefix_fn = compiler.parser.get_rule(prev.tp).prefix;

    match prefix_fn {
        Option::None => {
            emit_error(compiler, "Expect expression.");
            return;
        },
        Option::Some(func) => {
            func(compiler);
        },
    }

    loop {
        let infix_prec;
        let infix_fn;
        {
            let infix_rule = compiler.parser.get_rule(compiler.parser.current.tp);
            infix_prec = infix_rule.prec;
            infix_fn = infix_rule.infix;
        }

        if prec > infix_prec {
            break;
        }

        advance(compiler);

        match infix_fn {
            Option::None => {
                emit_error(compiler, "Expecting valid infix operator.");
            },
            Option::Some(func) => {
                func(compiler);
            }
        }
    }
}

fn parse_number(compiler: &mut Compiler) {
    let num: f64 = compiler.parser.previous.content.parse().expect("Can not parse number.");
    emit_constant(compiler, Value::DOUBLE { data: num });
}

fn parse_variable(compiler: &mut Compiler) {
    let vname: String = compiler.parser.previous.content.clone();
    let vid = make_str(compiler, vname);
    compiler.emit_inst(Inst::OP_GET_GLOBAL { name_idx: vid });
}

fn parse_string(compiler: &mut Compiler) {
    let s: String;
    {
        let s0 = &compiler.parser.previous.content;
        let sl = s0.len();
        s = s0[1..sl - 1].into();
    }
    emit_str(compiler, s);
}

fn parse_literal(compiler: &mut Compiler) {
    let tp = compiler.parser.previous.tp;

    match tp {
        TokenType::False => emit_constant(compiler, Value::BOOL { data: false }),
        TokenType::True => emit_constant(compiler, Value::BOOL { data: true }),
        TokenType::Nil => emit_constant(compiler, Value::NIL),
        _ => {},
    }
}

fn parse_grouping(compiler: &mut Compiler) {
    parse_expression(compiler);
    consume(compiler, TokenType::RightParen, "Expecting ')' after expression.".into());
}

fn parse_unary(compiler: &mut Compiler) {
    let tp = compiler.parser.previous.tp;

    parse_expression(compiler);

    match tp {
        TokenType::Minus => {
            compiler.emit_inst(Inst::OP_NEGATE);
        },
        TokenType::Bang => {
            compiler.emit_inst(Inst::OP_NOT);
        },
        _ => {
            panic!("Unexpected unary operator: {:?}", tp);
        }
    }
}

fn parse_binary(compiler: &mut Compiler) {
    let op_type = compiler.parser.previous.tp;
    let prec = compiler.parser.get_rule(op_type).prec;

    parse_prec(compiler, prec.succ());

    match op_type {
        TokenType::Plus => {
            compiler.emit_inst(Inst::OP_ADD);
        },
        TokenType::Minus => {
            compiler.emit_inst(Inst::OP_SUB);
        },
        TokenType::Star => {
            compiler.emit_inst(Inst::OP_MUL);
        },
        TokenType::Slash => {
            compiler.emit_inst(Inst::OP_DIV);
        },
        TokenType::EqualEqual => {
            compiler.emit_inst(Inst::OP_EQ);
        },
        TokenType::Greater => {
            compiler.emit_inst(Inst::OP_GT);
        },
        TokenType::Less => {
            compiler.emit_inst(Inst::OP_LT);
        },
        _ => { /* unreachable */ },
    }
}

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary
}

impl Precedence {
    pub fn succ(self) -> Self {
        match self {
            Precedence::None => Precedence::Assignment,
            Precedence::Assignment => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => Precedence::Primary,
        }
    }
}

pub type ParseFn = fn(&mut Compiler) -> ();

pub struct ParseRule {
    prefix: Option<ParseFn>,
    infix: Option<ParseFn>,
    prec: Precedence,
}

pub type ParseTable = HashMap<TokenType, ParseRule>;

impl ParseRule {
    pub fn new(prefix: Option<ParseFn>, infix: Option<ParseFn>, prec: Precedence) -> ParseRule {
        ParseRule { prefix, infix, prec }
    }

    pub fn make_rules() -> ParseTable {
        let mut m = HashMap::new();

        m.insert(TokenType::LeftParen, ParseRule::new(Some(parse_grouping), None, Precedence::None));
        m.insert(TokenType::RightParen, ParseRule::new(None, None, Precedence::None));
        m.insert(TokenType::LeftBrace, ParseRule::new(None, None, Precedence::None));
        m.insert(TokenType::RightBrace, ParseRule::new(None, None, Precedence::None));
        m.insert(TokenType::Comma, ParseRule::new(None, None, Precedence::None));
        m.insert(TokenType::Dot, ParseRule::new(None, None, Precedence::None));
        m.insert(TokenType::SemiColon, ParseRule::new(None, None, Precedence::None));
        m.insert(TokenType::Print, ParseRule::new(None, None, Precedence::None));

        m.insert(TokenType::Minus, ParseRule::new(Some(parse_unary), Some(parse_binary), Precedence::Term));
        m.insert(TokenType::Plus, ParseRule::new(None, Some(parse_binary), Precedence::Term));
        m.insert(TokenType::Slash, ParseRule::new(None, Some(parse_binary), Precedence::Factor));
        m.insert(TokenType::Star, ParseRule::new(None, Some(parse_binary), Precedence::Factor));

        m.insert(TokenType::EqualEqual, ParseRule::new(None, Some(parse_binary), Precedence::Equality));
        m.insert(TokenType::Greater, ParseRule::new(None, Some(parse_binary), Precedence::Comparison));
        m.insert(TokenType::Less, ParseRule::new(None, Some(parse_binary), Precedence::Comparison));

        m.insert(TokenType::Number, ParseRule::new(Some(parse_number), None, Precedence::None));
        m.insert(TokenType::Identifier, ParseRule::new(Some(parse_variable), None, Precedence::None));
        m.insert(TokenType::String, ParseRule::new(Some(parse_string), None, Precedence::None));
        m.insert(TokenType::False, ParseRule::new(Some(parse_literal), None, Precedence::None));
        m.insert(TokenType::True, ParseRule::new(Some(parse_literal), None, Precedence::None));
        m.insert(TokenType::Nil, ParseRule::new(Some(parse_literal), None, Precedence::None));

        m.insert(TokenType::Bang, ParseRule::new(Some(parse_unary), None, Precedence::None));


        m.insert(TokenType::EOF, ParseRule::new(None, None, Precedence::None));

        // TODO finish parsing table

        m
    }
}

