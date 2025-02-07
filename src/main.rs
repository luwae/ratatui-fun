mod debug;
use debug::{debug_print, debug_println};
mod maze;
mod tile;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use tile::{AlphaTileMap, TileMap};

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

#[derive(Debug, Default, Copy, Clone)]
pub enum BackgroundTile {
    #[default]
    Free,
    Wall,
}

impl From<&BackgroundTile> for ratatui::style::Color {
    fn from(value: &BackgroundTile) -> Self {
        match value {
            BackgroundTile::Free => Color::Black,
            BackgroundTile::Wall => Color::DarkGray,
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub enum VisitedTile {
    #[default]
    Visited,
}

impl From<&VisitedTile> for ratatui::style::Color {
    fn from(_value: &VisitedTile) -> Self {
        Color::Blue
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub enum ForegroundTile {
    #[default]
    Stack,
    Robot,
}

impl From<&ForegroundTile> for ratatui::style::Color {
    fn from(value: &ForegroundTile) -> Self {
        match value {
            ForegroundTile::Stack => Color::Yellow,
            ForegroundTile::Robot => Color::Green,
        }
    }
}

#[derive(Debug)]
pub struct App {
    exit: bool,
    layer_bg: TileMap<BackgroundTile>,
    layer_visited: AlphaTileMap<VisitedTile>,
    layer_fg: AlphaTileMap<ForegroundTile>,
    robot_pos: Pos,
    robot_dir: Direction,
    robot_stack: Vec<Pos>,
}

impl App {
    fn reinit(&mut self) {
        let (w, h) = (16, 16);
        let (pw, ph) = (2 * w + 1, 2 * h + 1);
        let maze = Maze::kruskal(w, h);
        let mut map = TileMap::with_default(pw as u16, ph as u16);
        for cy in 0..ph {
            for cx in 0..pw {
                map[Pos::new(cx, cy).into()] = match maze.tiles[cy][cx] {
                    maze::Tile::Free => BackgroundTile::Free,
                    maze::Tile::Wall => BackgroundTile::Wall,
                };
            }
        }
        self.layer_bg = map;
        self.layer_visited = AlphaTileMap::empty(pw as u16, ph as u16);
        self.layer_visited[(1, 1)] = Some(VisitedTile::Visited);
        self.layer_fg = AlphaTileMap::empty(pw as u16, ph as u16);
        self.layer_fg[(1, 1)] = Some(ForegroundTile::Robot);
        self.robot_pos = Pos::new(1, 1);
        self.robot_dir = Direction::E;
        self.robot_stack = Vec::new();
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

    fn draw(&mut self, frame: &mut Frame) {
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
                arr[idx] = match self.layer_bg[glob.into()] {
                    BackgroundTile::Free => b'.',
                    BackgroundTile::Wall => b'O',
                };
                idx += 1;
            }
        }
        arr
    }

    fn robot_step(&mut self) {
        let glob = self.robot_pos_with_offset((0, -1)).unwrap();
        // can only step into free fields
        match self.layer_bg[glob.into()] {
            BackgroundTile::Free => {
                if let Some(ForegroundTile::Robot) = self.layer_fg[self.robot_pos.into()] {
                    self.layer_fg[self.robot_pos.into()] = None;
                }
                self.robot_pos = glob;
                self.layer_fg[self.robot_pos.into()] = Some(ForegroundTile::Robot);
            }
            BackgroundTile::Wall => panic!("robot tried to move to wall at {}", glob),
        }
    }

    fn robot_stack_push(&mut self, pos: Pos) {
        self.robot_stack.push(pos);
        self.layer_fg[pos.into()] = Some(ForegroundTile::Stack);
    }

    fn robot_stack_pop(&mut self) -> Option<Pos> {
        if let Some(pos) = self.robot_stack.pop() {
            if let Some(ForegroundTile::Stack) = self.layer_fg[pos.into()] {
                self.layer_fg[pos.into()] = None;
            }
            Some(pos)
        } else {
            None
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

        let free = [
            front == b'.' && self.layer_visited[front_coords.into()].is_none(),
            right == b'.' && self.layer_visited[right_coords.into()].is_none(),
            left == b'.' && self.layer_visited[left_coords.into()].is_none(),
        ];
        if free[0] || free[1] || free[2] {
            match select_idx(&free[..]) {
                0 => {
                    debug_println("move front".to_string());
                    self.layer_visited[front_coords.into()] = Some(VisitedTile::Visited);
                    self.robot_stack_push(self.robot_pos);
                    self.robot_step();
                }
                1 => {
                    debug_println("move right".to_string());
                    self.layer_visited[right_coords.into()] = Some(VisitedTile::Visited);
                    self.robot_stack_push(self.robot_pos);
                    self.robot_turn_right();
                    self.robot_step();
                }
                2 => {
                    debug_println("move left".to_string());
                    self.layer_visited[left_coords.into()] = Some(VisitedTile::Visited);
                    self.robot_stack_push(self.robot_pos);
                    self.robot_turn_left();
                    self.robot_step();
                }
                _ => unreachable!(),
            }
        } else {
            debug_println("backtrack".to_string());
            // backtrack
            let back = match self.robot_stack_pop() {
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

fn select_idx(values: &[bool]) -> usize {
    let ntrue = values.iter().copied().filter(|t| *t).count();
    if ntrue == 0 {
        panic!("ntrue == 0");
    }
    let n = rand::random_range(0..ntrue);
    let mut m = 0;
    let mut idx = 0;
    loop {
        if values[idx] {
            if n == m {
                break idx;
            }
            m += 1;
        }
        idx += 1;
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
            .split(area);
        self.layer_bg.render(layout[1], buf);
        self.layer_visited.render(layout[1], buf);
        self.layer_fg.render(layout[1], buf);
    }
}

fn main() -> io::Result<()> {
    // println!("{}", maze);
    // return Ok(());
    let mut terminal = ratatui::init();
    let mut app = App {
        exit: false,
        layer_bg: TileMap::with_default(1, 1),
        layer_visited: AlphaTileMap::empty(1, 1),
        layer_fg: AlphaTileMap::empty(1, 1),
        robot_pos: Pos::new(1, 1),
        robot_dir: Direction::E,
        robot_stack: Vec::new(),
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

impl From<Pos> for (u16, u16) {
    fn from(value: Pos) -> Self {
        (value.x.try_into().unwrap(), value.y.try_into().unwrap())
    }
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
