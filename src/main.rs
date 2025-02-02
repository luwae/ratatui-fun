mod maze;

use std::io;

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

#[derive(Debug, Default)]
pub struct App {
    counter: u8,
    exit: bool,
    field: Vec<Vec<Tile>>,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
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

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
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
    };
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result
}
