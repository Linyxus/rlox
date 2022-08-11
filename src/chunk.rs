use crate::value::ValueArray;

#[derive(Debug)]
#[derive(Clone)]
pub enum KMethod {
    Print = 0,
}


#[derive(Debug)]
#[derive(Clone)]
pub enum Inst {
    RETURN,
    CONSTANT { idx: usize },
    OP_NEGATE,
    OP_ADD,
    OP_SUB,
    OP_MUL,
    OP_DIV,
    OP_NOT,
    OP_EQ,
    OP_GT,
    OP_LT,
    OP_KCALL { tp: KMethod },
    OP_POP,
    OP_DEFINE_GLOBAL { name_idx: usize },
    OP_GET_GLOBAL { name_idx: usize },
}

#[derive(Debug)]
pub struct Chunk {
    pub data: Vec<Inst>,
    pub value_array: ValueArray,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn write(&mut self, op: Inst, line: usize) {
        self.data.push(op);
        self.lines.push(line);
    }

    pub fn new() -> Chunk {
        Chunk {
            data: Vec::new(),
            value_array: ValueArray::new(),
            lines: Vec::new()
        }
    }
}

