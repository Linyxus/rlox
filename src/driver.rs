use crate::compiler::Compiler;
use crate::vm::{VM, InterpretResult};
use crate::debug;

pub struct Driver {
    debug_mode: bool,
}

impl Driver {
    pub fn new() -> Driver {
        Driver { debug_mode: false }
    }

    pub fn debug(&mut self) {
        self.debug_mode = true;
    }

    pub fn no_debug(&mut self) {
        self.debug_mode = false;
    }

    pub fn interpret(&self, source: String) -> InterpretResult {
        let mut compiler = Compiler::new(source);

        let comp_res = compiler.compile();

        if !comp_res {
            return InterpretResult::CompileError;
        }

        let mut vm = VM::new(compiler.current_chunk);

        if self.debug_mode {
            vm.trace_on();
            debug::disassemble_chunk(&vm.chunk, "code");
        }

        vm.run()
    }
}

