// Implements http://rosettacode.org/wiki/2048
//
// Based on the C++ version: http://rosettacode.org/wiki/2048#C.2B.2B
// Uses rustbox (termbox) to draw the board.

extern crate rustbox;
extern crate rand;

use std::fmt;
use rand::distributions::{IndependentSample, Range};
use rustbox::{Color, RustBox};
use rustbox::Key as RKey;

const NCOLS: usize = 5;
const NROWS: usize = 4;
const CELL_WIDTH: usize = 6;
const CELL_HEIGHT: usize = 3;
const BOARD_WIDTH: usize = 2 + (CELL_WIDTH + 2) * NCOLS;
const BOARD_HEIGHT: usize = 1 + (CELL_HEIGHT + 1) * NROWS;


#[derive(PartialEq, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn offset(self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Key {
    Right,
    Left,
    Up,
    Down,
    Char(char),
}

trait UI {
    fn wait_key(&self) -> Option<Key>;
    fn draw_bg(&self, x_offset: usize, y_offset: usize);
    fn draw_grid(&self, grid: [[Tile; NROWS]; NCOLS]);
    fn draw_tile(&self, col: usize, row: usize, tile: Tile);
    fn draw_tile_at(&self, tile: Tile, x_coord: usize, y_coord: usize);
    fn present(&self);
    fn draw_lost(&self);
    fn draw_won(&self);
    fn draw_score(&self, text: String);
    fn draw_instructions(&self, text: String);
}

struct TermboxUI<'a> {
    rustbox: &'a RustBox,
    board: [[Color; BOARD_HEIGHT]; BOARD_WIDTH],
}

impl<'a> UI for TermboxUI<'a> {
    fn wait_key(&self) -> Option<Key> {
        match self.rustbox.poll_event(false) {
            Ok(rustbox::Event::KeyEvent(key)) => {
                match key {
                    RKey::Char('q') => Some(Key::Char('q')),
                    RKey::Up => Some(Key::Up),
                    RKey::Down => Some(Key::Down),
                    RKey::Left => Some(Key::Left),
                    RKey::Right => Some(Key::Right),
                    _ => None,
                }
            }
            Err(e) => panic!("{}", e),
            _ => None,
        }
    }

    fn draw_bg(&self, x_offset: usize, y_offset: usize) {
        for x in 0 .. BOARD_WIDTH {
            for y in 0 .. BOARD_HEIGHT {
                let color = self.board[x][y];
                self.rustbox.print_char(x + x_offset,
                                   y + y_offset,
                                   rustbox::RB_NORMAL,
                                   color,
                                   color,
                                   ' ');
            }
        }
    }

    fn draw_grid(&self, grid: [[Tile; NROWS]; NCOLS]) {
        for x in 0.. NCOLS {
            for y in 0.. NROWS {
                self.draw_tile(x, y, grid[x][y])
            }
        }
    }

    fn draw_tile(&self, col: usize, row: usize, tile: Tile) {
        let x_offset = 2;
        let y_offset = 3;

        let x_coord = x_offset + col * CELL_WIDTH + col * 2;
        let y_coord = y_offset + row * CELL_HEIGHT + row;

        if tile.is_visible() {
            self.draw_tile_at(tile, x_coord, y_coord);
        }
    }

    fn draw_tile_at(&self, tile: Tile, x_coord: usize, y_coord: usize) {
        let x_text_offset = (CELL_WIDTH as f64 / 2 as f64).floor() as usize;
        let y_text_offset = (CELL_HEIGHT as f64 / 2 as f64).floor() as usize;

        let num: String = format!("{}", tile);
        let x_text_offset = x_text_offset - num.len() / 4;
        let tile_colour = match num.as_ref() {
            "2" => Color::Byte(224),
            "4" => Color::Byte(222),
            "8" => Color::Byte(216),
            "16" => Color::Byte(209),
            "32" => Color::Byte(202),
            "64" => Color::Byte(203),
            "128" => Color::Byte(230),
            "256" => Color::Byte(226),
            "512" => Color::Byte(193),
            "1024" => Color::Byte(190),
            "2048" => Color::Byte(214),
            _ => Color::Black,
        };
        if num != "0" {
            self.draw_rectangle(x_coord,
                                y_coord,
                                CELL_WIDTH,
                                CELL_HEIGHT,
                                tile_colour,
            );
            self.rustbox.print(x_coord + x_text_offset,
                               y_coord + y_text_offset,
                               rustbox::RB_NORMAL,
                               Color::Byte(232),
                               tile_colour,
                               &num);
        }
    }

    fn present(&self) {
        self.rustbox.present();
    }

    fn draw_lost(&self) {
        self.draw_text(16, 12, "You lost!".to_string(), Color::Red, Color::Black);
    }

    fn draw_won(&self) {
        self.draw_text(16, 12, "You won!".to_string(), Color::Green, Color::Black);
    }

    fn draw_score(&self, text: String) {
        self.draw_text(13, 1, text, Color::White, Color::Black);
    }

    fn draw_instructions(&self, text: String) {
        self.draw_text(11, 19, text, Color::White, Color::Black);
    }
}

impl<'a> TermboxUI<'a> {
    fn new(rustbox: &'a rustbox::RustBox) -> TermboxUI<'a> {

        let mut board = [[Color::Byte(137); BOARD_HEIGHT]; BOARD_WIDTH];

        for i in 0..NCOLS {
            for j in 0..NROWS {
                let left = 2 + i * (CELL_WIDTH + 2);
                let top = 1 + j * (CELL_HEIGHT + 1);
                if left + CELL_WIDTH < BOARD_WIDTH && top + CELL_HEIGHT < BOARD_HEIGHT {
                    for x in left .. left + CELL_WIDTH {
                        for y in top .. top + CELL_HEIGHT{
                            board[x][y] = Color::Byte(180);
                        }
                    }
                }
            }
        }
        TermboxUI {
            rustbox: rustbox,
            board: board,
        }
    }

    fn fill_area(&self, x: usize, y: usize, w: usize, h: usize, fg: Color, bg: Color) {
        for row in 0..h {
            for column in 0..w {
                self.rustbox.print_char(x + column, y + row, rustbox::RB_NORMAL, fg, bg, ' ');
            }
        }
    }

    fn draw_rectangle(&self,
                      x: usize,
                      y: usize,
                      w: usize,
                      h: usize,
                      fill: Color,
    ) {
        self.fill_area(x, y, w, h, fill, fill);
    }

    fn draw_text(&self, x: usize, y: usize, line: String, fg: Color, bg: Color) -> (usize, usize) {
        for (i, ch) in line.chars().enumerate() {
            self.rustbox.print_char(x + i, y, rustbox::RB_NORMAL, fg, bg, ch);
        }
        (x + line.len(), y)
    }
}

#[derive(Copy, Clone)]
struct Tile {
    _value: usize,
    _blocked: bool,
    _visible: bool,
}

impl Tile {
    fn new() -> Tile {
        Tile {
            _value: 0,
            _blocked: false,
            _visible: true,
        }
    }

    fn set(&mut self, val: usize) {
        self._value = val;
    }

    fn get(&self) -> usize {
        self._value
    }

    fn is_empty(&self) -> bool {
        self._value == 0
    }

    fn blocked(&mut self, b: bool) {
        self._blocked = b;
    }

    fn is_blocked(&self) -> bool {
        return self._blocked;
    }

    fn set_visible(&mut self, v: bool) {
        self._visible = v;
    }

    fn is_visible(&self) -> bool {
        return self._visible;
    }

}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self._value)
    }
}

impl PartialEq for Tile {
    fn eq(&self, other: &Tile) -> bool {
        self._value == other._value
    }

    fn ne(&self, other: &Tile) -> bool {
        self._value != other._value
    }
}

#[derive(PartialEq, Debug)]
enum State {
    Playing,
    Won,
    Lost,
}

struct Point {
    x: usize,
    y: usize,
}

struct Movement {
    tile: Tile,
    pold: Point,
    pnew: Point,
}

struct Game<'a> {
    ui: &'a UI,
    grid: [[Tile; NROWS]; NCOLS],
    state: State,
    score: usize,
    moved: bool,
    /// Vector containing tiles and their original position and destination
    tiles_moving: Vec<Movement>,
}

impl<'a> Game<'a> {
    fn new(ui: &'a UI) -> Game<'a> {
        Game {
            ui: ui,
            grid: [[Tile::new(); NROWS]; NCOLS],
            state: State::Playing,
            score: 0,
            moved: false,
            tiles_moving: Vec::new(),
        }
    }

    fn run(&mut self) {
        self.ui.draw_instructions("←,↑,→,↓ or q".to_string());

        for _ in 0..2 {
            self.add_tile();
        }

        loop {
            self.draw();
            self.moved = false;

            let key = self.ui.wait_key();
            if key == Some(Key::Char('q')) {
                break;
            }

            if self.state != State::Lost && self.state != State::Won {
                if let Some(direc) = match key {
                    Some(Key::Up) => Some(Direction::Up),
                    Some(Key::Down) => Some(Direction::Down),
                    Some(Key::Left) => Some(Direction::Left),
                    Some(Key::Right) => Some(Direction::Right),
                    _ => None,
                } {
                    self.move_all(direc);
                }
            }

            for i in 0.. NCOLS {
                for j in 0.. NROWS {
                    self.grid[i][j].blocked(false);
                }
            }

            if self.moved {
                self.add_tile();
            } else if !self.can_move() {
                self.state = State::Lost;
            }
        }
    }

    fn add_tile(&mut self) {
        let mut cantadd = true;
        'OUTER: for i in 0.. NCOLS {
            for j in 0.. NROWS {
                if self.grid[i][j].is_empty() {
                    cantadd = false;
                    break 'OUTER;
                }
            }
        }

        let cantmove = !self.can_move();
        if cantadd || cantmove {
            return;
        }

        let between = Range::new(0f64, 1.);
        let mut rng = rand::thread_rng();
        let a = between.ind_sample(&mut rng);

        let mut cell1 = rand::random::<(usize, usize)>();
        while !self.grid[cell1.0 % NCOLS][cell1.1 % NROWS].is_empty() {
            cell1 = rand::random::<(usize, usize)>();
        }
        let x = cell1.0 % NCOLS;
        let y = cell1.1 % NROWS;
        self.grid[x][y].set(if a > 0.9 { 4 } else { 2 });
    }

    fn can_move(&self) -> bool {
        for i in 0..NCOLS {
            for j in 0..NROWS {
                if self.grid[i][j].is_empty() {
                    return true;
                }

                if self.test_add(i + 1, j, self.grid[i][j]) {
                    return true;
                };
                if i > 0 && self.test_add(i - 1, j, self.grid[i][j]) {
                    return true;
                };
                if self.test_add(i, j + 1, self.grid[i][j]) {
                    return true;
                };
                if j > 0 && self.test_add(i, j - 1, self.grid[i][j]) {
                    return true;
                };
            }
        }

        return false;
    }

    fn test_add(&self, x: usize, y: usize, v: Tile) -> bool {
        if x > 3 || y > 3 {
            return false;
        }
        return self.grid[x][y] == v;
    }

    fn add_score(&mut self, score: usize) {
        self.score += score;

        if score == 2048 {
            self.state = State::Won;
        }
    }

    fn process_moving(&mut self) {
        for m in &self.tiles_moving {
            self.ui.draw_tile(m.pnew.x, m.pnew.y,
                              self.grid[m.pnew.x][m.pnew.y]);
        }
        self.ui.present();
        self.tiles_moving.truncate(0);
    }

    fn draw(&self) {
        self.ui.draw_score(format!("Score: {}", self.score));
        self.ui.draw_bg(0, 2);
        self.ui.draw_grid(self.grid);

        if self.state == State::Lost {
            self.ui.draw_lost();
        } else if self.state == State::Won {
            self.ui.draw_won();
        }

        self.ui.present();
    }

    fn move_direction(&mut self, x: usize, y: usize, d: Direction) -> (usize, usize) {
        let (xd, yd) = d.clone().offset();

        let xnew: i32 = x as i32 + xd;
        let ynew: i32 = y as i32 + yd;

        if ynew < 0 || ynew > (NROWS - 1) as i32 ||
            xnew < 0 || xnew > (NCOLS - 1) as i32 {
            return (x, y);
        }

        let xnew: usize = xnew as usize;
        let ynew: usize = ynew as usize;

        if !self.grid[xnew][ynew].is_empty() && self.grid[xnew][ynew] == self.grid[x][y] &&
            !self.grid[x][y].is_blocked() && !self.grid[xnew][ynew].is_blocked() {
                self.grid[x][y].set(0);
                let val = self.grid[xnew][ynew].get();
                self.grid[xnew][ynew].set(val * 2);
                self.add_score(val * 2);
                self.grid[xnew][ynew].blocked(true);
                self.moved = true;
            }
        else if self.grid[xnew][ynew].is_empty() && !self.grid[x][y].is_empty() {
                let val = self.grid[x][y].get();
                self.grid[xnew][ynew].set(val);
                self.grid[x][y].set(0);
                self.moved = true;
        }

        self.move_direction(xnew, ynew, d)
    }

    fn move_all(&mut self, direc: Direction) {
        for i in 0.. NCOLS {
            for j in 0.. NROWS {
                let tile = self.grid[i][j];
                if !tile.is_empty() {
                    let (inew, jnew) = self.move_direction(i, j, direc);
                    if inew != i || jnew != j {
                        self.tiles_moving.push(Movement {
                            tile: tile,
                            pold: Point { x: i, y: j},
                            pnew: Point { x: inew, y: jnew},
                        });
                    }
                }
            }
        }
    }
}

fn main() {
    let rustbox = match RustBox::init(
        rustbox::InitOptions {
            input_mode: rustbox::InputMode::Current,
            output_mode: rustbox::OutputMode::EightBit,
            buffer_stderr: true,
        }) {
        Result::Ok(v) => v,
        Result::Err(e) => panic!("{}", e),
    };

    let ui = TermboxUI::new(&rustbox);
    let mut game = Game::new(&ui);
    game.run();
}
