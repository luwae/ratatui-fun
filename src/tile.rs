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

impl<T> ratatui::widgets::Widget for &TileMap<T>
where
    for<'a> &'a T: Into<Color>,
{
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if area.width >= 2 * self.width && area.height >= self.height {
            for cy in 0..self.height {
                for cx in 0..self.width {
                    let tile = &self[(cx, cy)];
                    buf[(area.x + 2 * cx, area.y + cy)].set_bg(tile.into());
                    buf[(area.x + 2 * cx + 1, area.y + cy)].set_bg(tile.into());
                }
            }
        } else {
            for cy in 0..area.height {
                for cx in 0..area.width {
                    buf[(area.x + cx, area.y + cy)].set_bg(Color::Red);
                }
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
        if area.width >= 2 * self.0.width && area.height >= self.0.height {
            for cy in 0..self.0.height {
                for cx in 0..self.0.width {
                    if let Some(tile) = &self[(cx, cy)] {
                        buf[(area.x + 2 * cx, area.y + cy)].set_bg(tile.into());
                        buf[(area.x + 2 * cx + 1, area.y + cy)].set_bg(tile.into());
                    }
                }
            }
        } else {
            for cy in 0..area.height {
                for cx in 0..area.width {
                    buf[(area.x + cx, area.y + cy)].set_bg(Color::Red);
                }
            }
        }
    }
}
