mod debug;
use debug::{debug_print, debug_println};
mod maze;
mod tile;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use tile::TileMap;

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

#[derive(Debug, Default, Copy, Clone)]
pub enum MyTile {
    #[default]
    Free,
    Wall,
    Visited,
    Stack,
    Robot,
}

impl From<MyTile> for ratatui::style::Color {
    fn from(value: MyTile) -> Self {
        match value {
            MyTile::Free => Color::Black,
            MyTile::Wall => Color::DarkGray,
            MyTile::Visited => Color::Blue,
            MyTile::Stack => Color::Yellow,
            MyTile::Robot => Color::Green,
        }
    }
}

#[derive(Debug)]
pub struct App {
    exit: bool,
    map: TileMap<MyTile>,
    robot_pos: Pos,
    robot_dir: Direction,
    robot_stack: Vec<Pos>,
    robot_visited: Vec<Pos>,
}

impl App {
    fn reinit(&mut self) {
        let (w, h) = (16, 16);
        let maze = Maze::kruskal(w, h);
        let mut map = TileMap::with_size((2 * w + 1) as u16, (2 * h + 1) as u16);
        for cy in 0..h {
            for cx in 0..w {
                *map.get_mut(cx as u16, cy as u16).unwrap() = match maze.tiles[cy][cx] {
                    maze::Tile::Free => MyTile::Free,
                    maze::Tile::Wall => MyTile::Wall,
                };
            }
        }
        self.map = map;
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
                arr[idx] = match self.map.get(glob.x as u16, glob.y as u16).unwrap() {
                    MyTile::Free => b'.',
                    _ => b'O',
                };
                idx += 1;
            }
        }
        arr
    }

    fn robot_step(&mut self) {
        let glob = self.robot_pos_with_offset((0, -1)).unwrap();
        // can only step into free fields
        match self.map.get(glob.x as u16, glob.y as u16).unwrap() {
            MyTile::Free => {
                self.robot_pos = glob;
            }
            _ => panic!("robot tried to move to non-free {}", glob),
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
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
            .split(area);

        self.map.render(layout[1], buf);
    }
}

fn main() -> io::Result<()> {
    // println!("{}", maze);
    // return Ok(());
    let mut terminal = ratatui::init();
    let mut app = App {
        exit: false,
        map: TileMap::with_size(1, 1), // this gets reinit()ed anyways
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
