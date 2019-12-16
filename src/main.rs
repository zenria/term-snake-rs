#![allow(unused_must_use)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

use std::io;
use std::io::{Error, Read, Stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{async_stdin, clear};

mod utils;
use std::ops::Sub;
use std::time::{Duration, Instant};
use termion::color::*;
use utils::*;

#[derive(Copy, Debug, Clone)]
enum Direction {
    Up,
    Down,
    Right,
    Left,
}
#[derive(Copy, Debug, Clone)]
enum SnakeType {
    Body,
    Head,
}

#[derive(Copy, Debug, Clone)]
enum Cell {
    Empty,
    Food(u16),
    Snake(Direction, SnakeType),
}
#[derive(Copy, Debug, Clone)]
struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    fn inc(&mut self, direction: Direction, amount: u16) {
        match direction {
            Direction::Up => self.y -= 1,
            Direction::Down => self.y += 1,
            Direction::Right => self.x += 1,
            Direction::Left => self.x -= 1,
        }
    }
}

struct Screen {
    content: Vec<Cell>,
    out: RawTerminal<Stdout>,
    w: u16,
    h: u16,
}

impl Screen {
    fn new(w: u16, h: u16, out: RawTerminal<Stdout>) -> Self {
        Self {
            w,
            h,
            out,
            content: vec![Cell::Empty; (w + 1) as usize * (h + 1) as usize],
        }
    }

    fn get_index(&self, position: Position) -> usize {
        (position.x + position.y * self.w) as usize
    }

    fn set(&mut self, position: Position, cell: Cell) {
        let idx = self.get_index(position);
        self.content[idx] = cell;
        match &cell {
            Cell::Empty => {
                self.out.print(Bg(termion::color::Reset));
                set_pos(&mut self.out, position.x, position.y);
                self.out.print(" ");
            }
            Cell::Food(n) => {
                self.out.print(Fg(termion::color::LightMagenta));
                set_pos(&mut self.out, position.x, position.y);
                self.out.print(format!("{}", n));
            }
            Cell::Snake(_, snake_type) => {
                self.out.print(Fg(termion::color::LightYellow));
                set_pos(&mut self.out, position.x, position.y);
                match snake_type {
                    SnakeType::Body => self.out.print("O"),
                    SnakeType::Head => self.out.print("#"),
                }
            }
        }
        self.out.flush();
    }

    fn set_food_random(&mut self) {
        let food = rand::random::<u16>() % 9 + 1;
        loop {
            let x = rand::random::<u16>() % (self.w - 2) + 2;
            let y = rand::random::<u16>() % (self.h - 2) + 2;
            if let Cell::Empty = self.get(Position { x, y }) {
                self.set(Position { x, y }, Cell::Food(food));
                return;
            }
        }
    }

    fn get(&self, position: Position) -> Cell {
        self.content[self.get_index(position)]
    }

    fn display_score(&mut self, score: u32) {
        self.out.print(Bg(termion::color::Reset));
        self.out.print(Fg(termion::color::LightGreen));
        set_pos(&mut self.out, 3, 1);
        self.out.print(format!(" Score: {} ", score));
    }
}

enum GameStatus {
    Continue(u16),
    GameOver,
}
struct Game {
    head: Position,
    tail: Position,
    growing: u16,
    direction: Direction,
    screen: Screen,
    score: u32,
}

impl Game {
    fn new(w: u16, h: u16, out: RawTerminal<Stdout>) -> Self {
        // init
        let mut screen = Screen::new(w, h, out);
        let center = Position { x: w / 2, y: h / 2 };
        screen.set(center, Cell::Snake(Direction::Right, SnakeType::Head));
        for i in 0..20 {
            screen.set_food_random();
        }
        Self {
            head: center,
            tail: center,
            growing: 5,
            direction: Direction::Right,
            score: 0,
            screen,
        }
    }
    fn tick(&mut self, new_direction: Option<Direction>) -> GameStatus {
        if self.growing > 0 {
            // growing: do not move tail
            self.growing -= 1;
        } else {
            // not growing, move tail
            let tail_direction = self.screen.get(self.tail);
            if let Cell::Snake(tail_direction, _) = tail_direction {
                // clear tail
                self.screen.set(self.tail, Cell::Empty);
                // move tail
                self.tail.inc(tail_direction, 1);
            } else {
                panic!("Screen is not synchronized with Game state!");
            }
        }
        if let Some(direction) = new_direction {
            self.direction = direction;
        }

        // convert head to body
        self.screen
            .set(self.head, Cell::Snake(self.direction, SnakeType::Body));
        // Move head
        self.head.inc(self.direction, 1);

        let mut food = false;
        match self.screen.get(self.head) {
            Cell::Empty => (),
            Cell::Food(v) => {
                self.growing += v;
                self.score += v as u32;
                self.screen.set_food_random();
                self.screen.display_score(self.score);
                food = true;
            }
            Cell::Snake(_, _) => return GameStatus::GameOver,
        }

        self.screen
            .set(self.head, Cell::Snake(self.direction, SnakeType::Head));
        if self.head.x == 1
            || self.head.y == 1
            || self.head.x == self.screen.w
            || self.head.y == self.screen.h
        {
            GameStatus::GameOver
        } else {
            GameStatus::Continue(if food { 1 } else { 0 })
        }
    }
}

fn main() {
    println!("{}", clear::All);
    let (w, h) = termion::terminal_size().unwrap();

    let mut out = io::stdout().into_raw_mode().unwrap();

    let mut stdin = async_stdin().keys();

    out.print(termion::cursor::Hide);
    clear(&mut out);

    out.print(Fg(termion::color::Cyan));
    for x in 1..(w + 1) {
        set_pos(&mut out, x, 1);
        out.print('#');
        set_pos(&mut out, x, h);
        out.print('#');
    }
    for y in 1..(h + 1) {
        set_pos(&mut out, 1, y);
        out.print('#');
        set_pos(&mut out, w, y);
        out.print('#');
    }
    out.flush();

    let mut new_direction = None;
    let mut game = Game::new(w, h, io::stdout().into_raw_mode().unwrap());
    let mut sleep = Duration::from_millis(200);
    loop {
        match stdin.next() {
            Some(key) => match key {
                Ok(key) => match key {
                    Key::Left => new_direction = Some(Direction::Left),
                    Key::Right => new_direction = Some(Direction::Right),
                    Key::Up => new_direction = Some(Direction::Up),
                    Key::Down => new_direction = Some(Direction::Down),
                    Key::Esc => break,
                    _ => {}
                },
                Err(_) => {}
            },

            None => (),
        }
        let now = Instant::now();
        match game.tick(new_direction) {
            GameStatus::Continue(growing) => {
                if let Some(new_sleep) = sleep.checked_sub(Duration::from_millis(growing as u64)) {
                    sleep = new_sleep;
                }
            }
            GameStatus::GameOver => {
                set_pos(&mut out, 1, 2);
                out.print(Bg(termion::color::LightRed));
                out.print(Fg(termion::color::Black));
                out.print("PERDU !!");
                break;
            }
        }
        new_direction = None;

        std::thread::sleep(sleep);
    }

    // Show the cursor again before we exit.
    out.print(termion::cursor::Show);
}
