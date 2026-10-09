use terminal_tools::canvas::{line, DrawUnchecked};
use line::Line as InternalLine;
use super::{Pos, Canvas};

pub use line::LineVariant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line(InternalLine);

impl Line {
    pub fn new(pos: Pos, len: usize, ch: char, variant: LineVariant) -> Self {
        Self(InternalLine::new(pos, len, ch, variant))
    }

    pub fn pos(&mut self) -> &mut Pos {
        &mut self.0.pos
    }

    pub fn len(&mut self) -> &mut usize {
        &mut self.0.len
    }

    pub fn ch(&mut self) -> &mut char {
        &mut self.0.ch
    }

    pub fn variant(&mut self) -> &mut LineVariant {
        &mut self.0.variant
    }

    pub unsafe fn draw_unchecked(&self, canvas: &mut Canvas) {
        unsafe {
            self.0.draw_unchecked(&mut canvas.data)
        }
    }
}