mod debug;
use debug::{debug_print, debug_println};
mod maze;

use std::fmt;
use std::fs;
use std::io;
use std::ops;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::style::StyledContent;
use maze::Maze;
use ratatui::text::Span;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
    DefaultTerminal, Frame,
};

#[derive(Debug, Copy, Clone)]
pub enum Tile {
    Free,
    Wall,
}

#[derive(Debug)]
pub struct App {
    exit: bool,
    field: Vec<Vec<Tile>>,
    robot_pos: Pos,
    robot_dir: Direction,
    robot_stack: Vec<Pos>,
    robot_visited: Vec<Pos>,
}

impl App {
    fn reinit(&mut self) {
        let maze = Maze::kruskal(16, 16);
        self.field = maze
            .tiles
            .iter()
            .map(|line| {
                line.iter()
                    .map(|tile| match tile {
                        maze::Tile::Free => Tile::Free,
                        maze::Tile::Wall => Tile::Wall,
                    })
                    .collect()
            })
            .collect();
        self.robot_pos = Pos::new(1, 1);
        self.robot_dir = Direction::E;
        self.robot_stack = Vec::new();
        self.robot_visited = vec![self.robot_pos];
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let tick_rate = Duration::from_millis(100);
        let mut last_tick = Instant::now();
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            self.handle_events(timeout)?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self, timeout: Duration) -> io::Result<()> {
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Right => self.on_tick(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn robot_pos_with_offset(&self, offset: (isize, isize)) -> Option<Pos> {
        self.robot_pos + RelPos::new(offset.0, offset.1, self.robot_dir)
    }

    fn robot_scan(&mut self) -> [u8; 9] {
        let mut arr = [0u8; 9];
        let mut idx = 0;
        for y_loc in -1..=1 {
            for x_loc in -1..=1 {
                let glob = self.robot_pos_with_offset((x_loc, y_loc)).unwrap();
                arr[idx] = match self.field[glob.y][glob.x] {
                    Tile::Free => b'.',
                    Tile::Wall => b'O',
                };
                idx += 1;
            }
        }
        arr
    }

    fn robot_step(&mut self) {
        let glob = self.robot_pos_with_offset((0, -1)).unwrap();
        // can only step into free fields
        match self.field[glob.y][glob.x] {
            Tile::Free => {
                self.robot_pos = glob;
            }
            Tile::Wall => panic!("robot tried to move to wall at {}", glob),
        }
    }

    fn robot_turn_right(&mut self) {
        self.robot_dir = self.robot_dir.right();
    }

    fn robot_turn_left(&mut self) {
        self.robot_dir = self.robot_dir.left();
    }

    fn on_tick(&mut self) {
        debug_println(format!("current position: {}", self.robot_pos));
        debug_println(format!("current orientation: {:?}", self.robot_dir));
        let scan = self.robot_scan();
        let right = scan[5];
        let front = scan[1];
        let left = scan[3];
        let front_coords = self.robot_pos_with_offset((0, -1)).unwrap();
        let left_coords = self.robot_pos_with_offset((-1, 0)).unwrap();
        let right_coords = self.robot_pos_with_offset((1, 0)).unwrap();

        if front == b'.' && !self.robot_visited.contains(&front_coords) {
            debug_println("move front".to_string());
            self.robot_stack.push(self.robot_pos);
            self.robot_visited.push(front_coords);
            self.robot_step();
        } else if right == b'.' && !self.robot_visited.contains(&right_coords) {
            debug_println("move right".to_string());
            self.robot_stack.push(self.robot_pos);
            self.robot_visited.push(right_coords);
            self.robot_turn_right();
            self.robot_step();
        } else if left == b'.' && !self.robot_visited.contains(&left_coords) {
            debug_println("move left".to_string());
            self.robot_stack.push(self.robot_pos);
            self.robot_visited.push(left_coords);
            self.robot_turn_left();
            self.robot_step();
        } else {
            debug_println("backtrack".to_string());
            // backtrack
            let back = match self.robot_stack.pop() {
                Some(it) => it,
                None => {
                    self.reinit();
                    return;
                }
            };
            while back != self.robot_pos_with_offset((0, -1)).unwrap() {
                self.robot_turn_right();
            }
            self.robot_step();
        }
    }

    fn draw_bg(&self, buf: &mut Buffer, pos: Pos, color: Color) {
        buf[ratatui::layout::Position {
            x: (2 * pos.x) as u16,
            y: pos.y as u16,
        }]
        .set_bg(color);
        buf[ratatui::layout::Position {
            x: (2 * pos.x + 1) as u16,
            y: pos.y as u16,
        }]
        .set_bg(color);
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for cy in 0..self.field.len() {
            for cx in 0..self.field[0].len() {
                self.draw_bg(
                    buf,
                    Pos::new(cx, cy),
                    match self.field[cy][cx] {
                        Tile::Free => Color::Black,
                        Tile::Wall => Color::DarkGray,
                    },
                );
            }
        }

        for pos in self.robot_visited.iter().copied() {
            self.draw_bg(buf, pos, Color::Blue);
        }
        for pos in self.robot_stack.iter().copied() {
            self.draw_bg(buf, pos, Color::Yellow);
        }

        self.draw_bg(buf, self.robot_pos, Color::Green);
    }
}

fn main() -> io::Result<()> {
    // println!("{}", maze);
    // return Ok(());
    let mut terminal = ratatui::init();
    let mut app = App {
        exit: false,
        field: Vec::new(),
        robot_pos: Pos::new(1, 1),
        robot_dir: Direction::E,
        robot_stack: Vec::new(),
        robot_visited: Vec::new(),
    };
    app.reinit();
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Direction {
    N,
    E,
    S,
    W,
}

impl Direction {
    fn right(self) -> Self {
        use Direction as D;
        match self {
            D::N => D::E,
            D::E => D::S,
            D::S => D::W,
            D::W => D::N,
        }
    }

    fn left(self) -> Self {
        use Direction as D;
        match self {
            D::N => D::W,
            D::E => D::N,
            D::S => D::E,
            D::W => D::S,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Pos {
    x: usize,
    y: usize,
}

impl Pos {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl ops::Add<RelPos> for Pos {
    type Output = Option<Pos>;

    fn add(self, mut rhs: RelPos) -> Option<Pos> {
        rhs = rhs.reorient(Direction::N);
        Some(Pos {
            x: (TryInto::<isize>::try_into(self.x).ok()? + rhs.x)
                .try_into()
                .ok()?,
            y: (TryInto::<isize>::try_into(self.y).ok()? + rhs.y)
                .try_into()
                .ok()?,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct RelPos {
    x: isize,
    y: isize,
    dir: Direction,
}

impl RelPos {
    fn new(x: isize, y: isize, dir: Direction) -> Self {
        Self { x, y, dir }
    }

    fn reorient_right(self) -> RelPos {
        Self::new(self.y, -self.x, self.dir.right())
    }

    fn reorient(mut self, new_dir: Direction) -> Self {
        while self.dir != new_dir {
            self = self.reorient_right();
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reorient() {
        use Direction as D;

        let rp = RelPos::new(-1, 0, D::N);
        assert_eq!(RelPos::new(0, 1, D::E), rp.reorient_right());
        let rp = RelPos::new(0, -1, D::N);
        assert_eq!(RelPos::new(-1, 0, D::E), rp.reorient_right());

        let rp = RelPos::new(-1, 0, D::E);
        assert_eq!(RelPos::new(0, 1, D::S), rp.reorient_right());
        let rp = RelPos::new(0, -1, D::E);
        assert_eq!(RelPos::new(-1, 0, D::S), rp.reorient_right());

        let rp = RelPos::new(5, -3, D::S);
        assert_eq!(RelPos::new(3, 5, D::E), rp.reorient(D::E));
    }
}
