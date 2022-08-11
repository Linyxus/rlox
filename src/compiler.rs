use crate::scanner::{TokenType, Token, ScannerState, next_token};
use crate::parser::ParserState;
use crate::parser;
use crate::chunk::{Inst, Chunk};

pub struct Compiler {
    pub source: String,
    pub scanner: ScannerState,
    pub parser: ParserState,
    pub current_chunk: Chunk,
}

impl Compiler {
    pub fn new(source: String) -> Compiler {
        Compiler {
            source,
            scanner: ScannerState::new(),
            parser: ParserState::new(),
            current_chunk: Chunk::new(),
        }
    }

    pub fn compile(&mut self) -> bool {
        // let mut line = 1 << 20;
        // loop {
        //     let tok = next_token(self);

        //     match tok.tp {
        //         TokenType::EOF => break,
        //         _ => {},
        //     }

        //     if tok.line != line {
        //         println!("{:04} {}", tok.line, tok.show());
        //         line = tok.line;
        //     } else {
        //         println!("   | {}", tok.show());
        //     }
        // }

        parser::advance(self);
        // parser::parse_expression(self);
        // parser::consume(self, TokenType::EOF, "Expecting end of expression.");
        while !parser::try_consume(self, TokenType::EOF) {
            parser::parse_decl(self);
        }

        self.emit_inst(Inst::RETURN);

        return !self.parser.had_error;
    }

    pub fn next_token(&mut self) -> Token {
        next_token(self)
    }

    pub fn emit_inst(&mut self, inst: Inst) {
        self.current_chunk.write(inst, self.parser.previous.line as usize);
    }
}
