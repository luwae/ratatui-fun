use std::ops;

use ratatui::style::Color;

#[derive(Debug)]
pub struct TileMap<T> {
    data: Vec<Vec<T>>,
    width: u16,
    height: u16,
}

impl<T> TileMap<T>
where
    T: Clone + Default,
{
    pub fn with_default(width: u16, height: u16) -> Self {
        let mut data = Vec::with_capacity(height as usize);
        for _ in 0..height {
            data.push(vec![Default::default(); width as usize]);
        }
        Self {
            data,
            width,
            height,
        }
    }
}

impl<T> ops::Index<(u16, u16)> for TileMap<T> {
    type Output = T;

    fn index(&self, index: (u16, u16)) -> &Self::Output {
        self.data
            .get(index.1 as usize)
            .unwrap()
            .get(index.0 as usize)
            .unwrap()
    }
}

impl<T> ops::IndexMut<(u16, u16)> for TileMap<T> {
    fn index_mut(&mut self, index: (u16, u16)) -> &mut Self::Output {
        self.data
            .get_mut(index.1 as usize)
            .unwrap()
            .get_mut(index.0 as usize)
            .unwrap()
    }
}

// const ARR_RIGHT: char = '⮕';
// const ARR_DOWN: char = '⬇';
// const ARR_DOWNRIGHT: char = '⬊';
// const ARR_RIGHT: char = '>';
// const ARR_DOWN: char = 'V';
// const ARR_DOWNRIGHT: char = '\\';
const ARR_RIGHT: char = ' ';
const ARR_DOWN: char = ' ';
const ARR_DOWNRIGHT: char = ' ';

impl<T> ratatui::widgets::Widget for &TileMap<T>
where
    for<'a> &'a T: Into<Color>,
{
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        for cy in 0..self.height {
            for cx in 0..self.width {
                let tile = &self[(cx, cy)];
                // TODO maybe prettier with an if let
                buf.cell_mut((area.x + 2 * cx, area.y + cy))
                    .map(|cell| cell.set_bg(tile.into()));
                buf.cell_mut((area.x + 2 * cx + 1, area.y + cy))
                    .map(|cell| cell.set_bg(tile.into()));
            }
        }
        if area.width < 2 * self.width {
            for y in area.top()..area.bottom() {
                buf[(area.right() - 2, y)]
                    .set_bg(Color::White)
                    .set_fg(Color::Black)
                    .set_char('-');
                buf[(area.right() - 1, y)]
                    .set_bg(Color::White)
                    .set_fg(Color::Black)
                    .set_char('>');
            }
        }
        if area.height < self.height {
            for x in area.left()..area.right() {
                buf[(x, area.bottom() - 1)]
                    .set_bg(Color::White)
                    .set_fg(Color::Black)
                    .set_char(if (x - area.left()) % 2 == 0 && x < area.right() - 2 {
                        'V'
                    } else {
                        ' '
                    });
            }
        }
    }
}

#[derive(Debug)]
pub struct AlphaTileMap<T>(TileMap<Option<T>>);

impl<T> AlphaTileMap<T>
where
    T: Clone,
{
    pub fn empty(width: u16, height: u16) -> Self {
        Self(TileMap::with_default(width, height))
    }
}

impl<T> ops::Index<(u16, u16)> for AlphaTileMap<T> {
    type Output = Option<T>;

    fn index(&self, index: (u16, u16)) -> &Self::Output {
        &self.0[index]
    }
}

impl<T> ops::IndexMut<(u16, u16)> for AlphaTileMap<T> {
    fn index_mut(&mut self, index: (u16, u16)) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<T> ratatui::widgets::Widget for &AlphaTileMap<T>
where
    for<'a> &'a T: Into<Color>,
{
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        for cy in 0..self.0.height {
            for cx in 0..self.0.width {
                if let Some(tile) = &self[(cx, cy)] {
                    // TODO maybe prettier with an if let
                    buf.cell_mut((area.x + 2 * cx, area.y + cy))
                        .map(|cell| cell.set_bg(tile.into()));
                    buf.cell_mut((area.x + 2 * cx + 1, area.y + cy))
                        .map(|cell| cell.set_bg(tile.into()));
                }
            }
        }
    }
}
