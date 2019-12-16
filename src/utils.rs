use std::fmt::Display;
use std::io::Stdout;
use std::io::Write;
use termion::raw::RawTerminal;

pub fn clear(out: &mut RawTerminal<Stdout>) {
    out.print(termion::clear::All);
}

pub fn set_pos(out: &mut RawTerminal<Stdout>, x: u16, y: u16) {
    out.print(termion::cursor::Goto(x, y));
}

pub trait Print {
    fn print<T: Display>(&mut self, to_print: T);
}

impl Print for RawTerminal<Stdout> {
    fn print<T: Display>(&mut self, to_print: T) {
        write!(self, "{}", to_print).unwrap();
    }
}
