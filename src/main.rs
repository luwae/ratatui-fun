mod maze;

use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::style::StyledContent;
use maze::Maze;
use ratatui::layout::Position;
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
    counter: u8,
    exit: bool,
    field: Vec<Vec<Tile>>,
    robot_pos: (usize, usize),
    robot_dir: Direction,
    robot_stack: Vec<(usize, usize)>,
    robot_visited: Vec<(usize, usize)>,
}

impl App {
    fn init(&mut self) {
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
        self.robot_pos = (1, 1);
        self.robot_dir = Direction::E;
        self.robot_stack = Vec::new();
        self.robot_visited = vec![self.robot_pos];
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let tick_rate = Duration::from_millis(20);
        let mut last_tick = Instant::now();
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            self.handle_events(timeout)?;

            if last_tick.elapsed() >= tick_rate {
                let done = self.on_tick();
                if done {
                    self.init();
                }
                last_tick = Instant::now();
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
        /*
        frame.render_widget(Block::bordered().title("Hey"), Rect::new(10, 10, 25, 25));
        let p = Paragraph::new("Hello, World!")
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Title")
                    .border_type(BorderType::Rounded),
            );
        frame.render_widget(p, Rect::new(40, 10, 20, 20));
        */
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
            KeyCode::Left => self.decrement_counter(),
            KeyCode::Right => self.increment_counter(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn increment_counter(&mut self) {
        self.counter += 1;
    }

    fn decrement_counter(&mut self) {
        self.counter -= 1;
    }

    fn local_to_global(&self, offset: (isize, isize)) -> (isize, isize) {
        use Direction as D;
        match self.robot_dir {
            D::N => offset,
            D::E => (-offset.1, offset.0),
            D::S => (-offset.0, -offset.1),
            D::W => (offset.1, -offset.0),
        }
    }

    fn global_to_local(&self, offset: (isize, isize)) -> (isize, isize) {
        use Direction as D;
        match self.robot_dir {
            D::N => offset,
            D::E => (offset.1, -offset.0),
            D::S => (-offset.0, -offset.1),
            D::W => (-offset.1, offset.0),
        }
    }

    fn as_seen_from_robot(&self, offset: (isize, isize)) -> (usize, usize) {
        let off_glob = self.local_to_global(offset);
        (
            (self.robot_pos.0 as isize + off_glob.0).try_into().unwrap(),
            (self.robot_pos.1 as isize + off_glob.1).try_into().unwrap(),
        )
    }

    fn robot_scan(&mut self) -> [u8; 9] {
        let mut arr = [0u8; 9];
        let mut idx = 0;
        for y_loc in -1..=1 {
            for x_loc in -1..=1 {
                let (x_glob, y_glob) = self.as_seen_from_robot((x_loc, y_loc));
                arr[idx] = match self.field[y_glob][x_glob] {
                    Tile::Free => b'.',
                    Tile::Wall => b'O',
                };
                idx += 1;
            }
        }
        arr
    }

    fn robot_step(&mut self) {
        let (x_glob, y_glob) = self.as_seen_from_robot((0, -1));
        // can only step into free fields
        match self.field[y_glob][x_glob] {
            Tile::Free => {
                self.robot_pos = (x_glob, y_glob);
            }
            Tile::Wall => panic!("robot tried to move to wall at ({}, {})", x_glob, y_glob),
        }
    }

    fn robot_turn_right(&mut self) {
        use Direction as D;
        self.robot_dir = match self.robot_dir {
            D::N => D::E,
            D::E => D::S,
            D::S => D::W,
            D::W => D::N,
        }
    }

    fn robot_turn_left(&mut self) {
        use Direction as D;
        self.robot_dir = match self.robot_dir {
            D::N => D::W,
            D::E => D::N,
            D::S => D::E,
            D::W => D::S,
        }
    }

    fn on_tick(&mut self) -> bool {
        let scan = self.robot_scan();
        let right = scan[5];
        let front = scan[1];
        let left = scan[3];
        let front_coords = self.as_seen_from_robot((0, -1));
        let left_coords = self.as_seen_from_robot((-1, 0));
        let right_coords = self.as_seen_from_robot((1, 0));

        if front == b'.' && !self.robot_visited.contains(&front_coords) {
            self.robot_stack.push(self.robot_pos);
            self.robot_visited.push(front_coords);
            self.robot_step();
        } else if right == b'.' && !self.robot_visited.contains(&right_coords) {
            self.robot_stack.push(self.robot_pos);
            self.robot_visited.push(right_coords);
            self.robot_turn_right();
            self.robot_step();
        } else if left == b'.' && !self.robot_visited.contains(&left_coords) {
            self.robot_stack.push(self.robot_pos);
            self.robot_visited.push(left_coords);
            self.robot_turn_left();
            self.robot_step();
        } else {
            // backtrack
            let back = match self.robot_stack.pop() {
                Some(it) => it,
                None => return true,
            };
            while back != self.as_seen_from_robot((0, -1)) {
                self.robot_turn_right();
            }
            self.robot_step();
        }
        false
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for cy in 0..self.field.len() {
            for cx in 0..self.field[0].len() {
                buf[Position {
                    x: (2 * cx) as u16,
                    y: cy as u16,
                }]
                .set_bg(match self.field[cy][cx] {
                    Tile::Free => Color::Black,
                    Tile::Wall => Color::DarkGray,
                });
                buf[Position {
                    x: (2 * cx + 1) as u16,
                    y: cy as u16,
                }]
                .set_bg(match self.field[cy][cx] {
                    Tile::Free => Color::Black,
                    Tile::Wall => Color::DarkGray,
                });
            }
        }

        for coord in &self.robot_visited {
            buf[Position {
                x: (2 * coord.0) as u16,
                y: coord.1 as u16,
            }]
            .set_bg(Color::Blue);
            buf[Position {
                x: (2 * coord.0 + 1) as u16,
                y: coord.1 as u16,
            }]
            .set_bg(Color::Blue);
        }
        for coord in &self.robot_stack {
            buf[Position {
                x: (2 * coord.0) as u16,
                y: coord.1 as u16,
            }]
            .set_bg(Color::Yellow);
            buf[Position {
                x: (2 * coord.0 + 1) as u16,
                y: coord.1 as u16,
            }]
            .set_bg(Color::Yellow);
        }

        buf[Position {
            x: (2 * self.robot_pos.0) as u16,
            y: self.robot_pos.1 as u16,
        }]
        .set_bg(Color::Green);
        buf[Position {
            x: (2 * self.robot_pos.0 + 1) as u16,
            y: self.robot_pos.1 as u16,
        }]
        .set_bg(Color::Green);
    }
}

fn main() -> io::Result<()> {
    // println!("{}", maze);
    // return Ok(());
    let mut terminal = ratatui::init();
    let mut app = App {
        counter: 0,
        exit: false,
        field: Vec::new(),
        robot_pos: (1, 1),
        robot_dir: Direction::E,
        robot_stack: Vec::new(),
        robot_visited: Vec::new(),
    };
    app.init();
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result
}

#[derive(Debug)]
enum Direction {
    N,
    E,
    S,
    W,
}
