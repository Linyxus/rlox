#[derive(Debug)]
#[derive(Clone, Copy)]
pub struct Span {
    start: usize,
    len: usize,
}

impl Span {
    pub fn new(start: usize, len: usize) -> Span {
        Span { start, len }
    }
}
