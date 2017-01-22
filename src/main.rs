// Implements http://rosettacode.org/wiki/2048
//
// Based on the C++ version: http://rosettacode.org/wiki/2048#C.2B.2B
// Uses rustbox (termbox) to draw the board.

extern crate rustbox;
extern crate rand;

use std::fmt;
use std::time;

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
    fn wait_key(&self, Option<u64>) -> Option<Key>;
    fn draw_bg(&self, x_offset: usize, y_offset: usize);
    fn draw_grid(&self, grid: [[Tile; NROWS]; NCOLS]);
    fn draw_tile(&self, col: usize, row: usize, tile: Tile, partial: Option<f32>);
    fn draw_tile_at(&self, tile: Tile, x_coord: usize, y_coord: usize, partial: Option<f32>);
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
    fn wait_key(&self, timeout: Option<u64>) -> Option<Key> {
        let event = match timeout {
            Some(time) => self.rustbox.peek_event(std::time::Duration::from_millis(time), false),
            None => self.rustbox.poll_event(false),
        };
        match event {
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
                self.draw_tile(x, y, grid[x][y], None)
            }
        }
    }

    fn draw_tile(&self, col: usize, row: usize, tile: Tile, partial: Option<f32>) {
        let x_offset = 2;
        let y_offset = 3;

        let x_coord = x_offset + col * CELL_WIDTH + col * 2;
        let y_coord = y_offset + row * CELL_HEIGHT + row;

        self.draw_tile_at(tile, x_coord, y_coord, partial);
    }

    fn draw_tile_at(&self, tile: Tile, x_coord: usize, y_coord: usize, partial: Option<f32>) {
        let x_text_offset = (CELL_WIDTH as f64 / 2 as f64).floor() as usize;
        let y_text_offset = (CELL_HEIGHT as f64 / 2 as f64).floor() as usize;
        let x_centre = x_coord + x_text_offset;
        let y_centre = y_coord + y_text_offset;

        let num: String = format!("{}", tile);
        let x_text_pos = x_centre - num.len() / 2;
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
            if let Some(ratio) = partial {
                for column in 0 .. CELL_WIDTH {
                    for row in 0 .. CELL_HEIGHT {
                        let x = x_coord + column;
                        let y = y_coord + row;
                        if (x as f32 - x_centre as f32).abs() < CELL_WIDTH as f32 * ratio / 2.0
                            || (y as f32 - y_centre as f32).abs() < CELL_HEIGHT as f32 * ratio / 2.0 {
                            self.rustbox.print_char(x, y,
                                                    rustbox::RB_NORMAL,
                                                    tile_colour,
                                                    tile_colour, ' ');
                        }
                    }
                }
            } else {
                self.draw_rectangle(x_coord,
                                    y_coord,
                                    CELL_WIDTH,
                                    CELL_HEIGHT,
                                    tile_colour,
                );
            }
            self.rustbox.print(x_text_pos,
                               y_centre,
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
    _value_old: usize,
    _blocked: bool,
    /// the tile changed, but the old value should be shown before animation is done
    _pending: bool,
}

impl Tile {
    fn new() -> Tile {
        Tile {
            _value: 0,
            _value_old: 0,
            _blocked: false,
            _pending: false,
        }
    }

    fn from_value(value: usize) -> Tile {
        Tile {
            _value: value,
            _value_old: 0,
            _blocked: false,
            _pending: false,
        }
    }

    fn set(&mut self, val: usize) {
        self._value_old = self._value;
        self._value = val;
    }

    fn get(&self) -> usize {
        if self._pending {
            self._value_old
        } else {
            self._value
        }
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

    fn set_pending(&mut self, pending: bool) {
        self._pending = pending;
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get())
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

struct Appearing {
    position: Point,
    value: usize,
}

struct Game<'a> {
    ui: &'a UI,
    grid: [[Tile; NROWS]; NCOLS],
    state: State,
    score: usize,
    moved: bool,
    /// Vector containing tiles and their original position and destination
    tiles_moving: Vec<Movement>,
    /// where new tiles are appearing
    points_appearing: Vec<Appearing>,
    /// The time when the latest movement started
    animation_start: time::Instant,
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
            points_appearing: Vec::new(),
            animation_start: time::Instant::now(),
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

            let key = if self.tiles_moving.len() > 0 {
                // when there are tiles waiting to be moved, wait for a short time
                self.ui.wait_key(Some(10))
            } else {
                self.ui.wait_key(None)
            };

            if key == Some(Key::Char('q')) {
                break;
            } else if key == None {
                continue;
            }

            // finish any on-going animation immediately
            self.finish_animation();

            // start moving
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
            self.animation_start = time::Instant::now();
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
        self.points_appearing.push(Appearing {
            value: if a > 0.9 { 4 } else { 2 },
            position: Point { x: cell1.0 % NCOLS, y: cell1.1 % NROWS},
        });
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

    fn finish_animation(&mut self) {
        for m in &self.tiles_moving {
            self.grid[m.pnew.x][m.pnew.y].set_pending(false);
        }
        self.tiles_moving.truncate(0);

        for a in &self.points_appearing {
            self.grid[a.position.x][a.position.y].set(a.value);
        }
        self.points_appearing.truncate(0);
    }

    fn get_progress(&self) -> f32 {
        // how much of the animation has been done
        // duration of the entire animation in milliseconds
        let animation_duration: u16 = 500;
        let elapsed: u16 = self.animation_start.elapsed().as_secs() as u16 * 1000
            + (self.animation_start.elapsed().subsec_nanos() / 1000000) as u16;
        elapsed as f32 / animation_duration as f32
    }

    fn draw_moving(&mut self) {
        let ratio = self.get_progress();
        if ratio > 0.99 {
            self.finish_animation();
            return;
        }
        for m in &self.tiles_moving {
            let col = m.pold.x as f32 + (m.pnew.x as f32 - m.pold.x as f32) * ratio;
            let row = m.pold.y as f32 + (m.pnew.y as f32 - m.pold.y as f32) * ratio;

            let x_offset = 2.0;
            let y_offset = 3.0;

            let x_now = x_offset + col * CELL_WIDTH as f32 + col * 2.0;
            let y_now = y_offset + row * CELL_HEIGHT as f32 + row;

            self.ui.draw_tile_at(m.tile, x_now as usize, y_now as usize, None);
        }

        for a in &self.points_appearing {
            let x_offset = 2.0;
            let y_offset = 3.0;
            let col = a.position.x as f32;
            let row = a.position.y as f32;

            let x = x_offset + col * CELL_WIDTH as f32 + col * 2.0;
            let y = y_offset + row * CELL_HEIGHT as f32 + row;

            self.ui.draw_tile_at(Tile::from_value(a.value),
                                 x as usize, y as usize,
                                 Some(ratio));
        }
    }

    fn draw(&mut self) {
        self.ui.draw_score(format!("Score: {}", self.score));
        self.ui.draw_bg(0, 2);

        self.draw_moving();

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

        let mut tilemoved = false;
        if !self.grid[xnew][ynew].is_empty() && self.grid[xnew][ynew] == self.grid[x][y] &&
            !self.grid[x][y].is_blocked() && !self.grid[xnew][ynew].is_blocked() {
                self.grid[x][y].set(0);
                let val = self.grid[xnew][ynew].get();
                self.grid[xnew][ynew].set(val * 2);
                self.add_score(val * 2);
                self.grid[xnew][ynew].blocked(true);
                self.moved = true;
                tilemoved = true;
            }
        else if self.grid[xnew][ynew].is_empty() && !self.grid[x][y].is_empty() {
            let val = self.grid[x][y].get();
            self.grid[xnew][ynew].set(val);
            self.grid[x][y].set(0);
            self.moved = true;
            tilemoved = true;
        }

        if tilemoved {
            self.move_direction(xnew, ynew, d)
        } else {
            (x, y)
        }
    }

    fn move_all(&mut self, direc: Direction) {
        for i in 0.. NCOLS {
            for j in 0.. NROWS {
                let tile = self.grid[i][j];
                if !tile.is_empty() {
                    let (inew, jnew) = self.move_direction(i, j, direc);
                    if inew != i || jnew != j {
                        self.grid[inew][jnew].set_pending(true);
                        self.tiles_moving.push(Movement {
                            // it's not grid[i][j], which may have changed
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
