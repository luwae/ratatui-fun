mod maze;

use std::io;
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
    counter: u8,
    exit: bool,
    field: Vec<Vec<Tile>>,
    robot_pos: (usize, usize),
    robot_dir: Direction,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let tick_rate = Duration::from_millis(100);
        let mut last_tick = Instant::now();
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            self.handle_events(timeout)?;

            if last_tick.elapsed() >= tick_rate {
                on_tick(self);
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

    fn robot_scan(&mut self) -> [u8; 9] {
        let mut arr = [0u8; 9];
        let mut idx = 0;
        for y_loc in -1..=1 {
            for x_loc in -1..=1 {
                let (xoff_glob, yoff_glob) = self.local_to_global((x_loc, y_loc));
                let (x_glob, y_glob): (usize, usize) = (
                    (self.robot_pos.0 as isize + xoff_glob).try_into().unwrap(),
                    (self.robot_pos.1 as isize + yoff_glob).try_into().unwrap(),
                );
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
        let (xoff_glob, yoff_glob) = self.local_to_global((0, -1));
        let (x_glob, y_glob): (usize, usize) = (
            (self.robot_pos.0 as isize + xoff_glob).try_into().unwrap(),
            (self.robot_pos.1 as isize + yoff_glob).try_into().unwrap(),
        );
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
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Counter App Tutorial ".bold());
        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);
        let counter_text = Text::from(vec![Line::from(vec![
            " Value ".into(),
            self.counter.to_string().yellow(),
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);

        let x = self.field[0].len();
        let y = self.field.len();
        let mut cx = 0;
        let mut cy = 0;
        let mut text: Vec<Line> = Vec::new();

        #[allow(clippy::explicit_counter_loop)]
        for line in &self.field {
            let mut linevec: Vec<Span> = Vec::new();
            for tile in line {
                linevec.push(if (cx, cy) == self.robot_pos {
                    "@".green().bold()
                } else {
                    match tile {
                        Tile::Free => ".".dark_gray(),
                        Tile::Wall => "O".white(),
                    }
                });
                cx += 1;
            }
            text.push(linevec.into());
            cx = 0;
            cy += 1;
        }

        /*
        let text = self
            .field
            .iter()
            .map(|line| {
                line.iter()
                    .map(|tile| match tile {
                        Tile::Free => ".".dark_gray(),
                        Tile::Wall => "O".white(),
                    })
                    .collect::<Vec<Span>>()
                    .into()
            })
            .collect::<Vec<Line>>();
        */

        Paragraph::new(text)
            .block(Block::bordered())
            .render(Rect::new(2, 2, (x + 2) as u16, (y + 2) as u16), buf);
    }
}

fn main() -> io::Result<()> {
    let maze = Maze::kruskal(16, 8);
    // println!("{}", maze);
    // return Ok(());
    let mut terminal = ratatui::init();
    let mut app = App {
        counter: 0,
        exit: false,
        field: maze
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
            .collect(),
        robot_pos: (1, 1),
        robot_dir: Direction::E,
    };
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

fn on_tick(app: &mut App) {
    let scan = app.robot_scan();
    let right = scan[5];
    let front = scan[1];
    if right == b'.' {
        app.robot_turn_right();
        app.robot_step();
    } else if front == b'.' {
        app.robot_step();
    } else {
        app.robot_turn_left()
    }
}
