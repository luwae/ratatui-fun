#[derive(Debug)]
pub struct TileMap<T> {
    data: Vec<Vec<T>>,
    width: u16,
    height: u16,
}

impl<T> TileMap<T>
where
    T: Into<ratatui::style::Color> + Default + Copy + Clone,
{
    pub fn with_size(width: u16, height: u16) -> Self {
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

    pub fn get(&self, x: u16, y: u16) -> Option<&T> {
        self.data.get(y as usize)?.get(x as usize)
    }

    pub fn get_mut(&mut self, x: u16, y: u16) -> Option<&mut T> {
        self.data.get_mut(y as usize)?.get_mut(x as usize)
    }
}

impl<T> ratatui::widgets::Widget for &TileMap<T>
where
    T: Into<ratatui::style::Color> + Default + Copy + Clone,
{
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if area.width >= 2 * self.width && area.height >= self.height {
            for cy in 0..self.height {
                for cx in 0..self.width {
                    let tile = *self.get(cx, cy).unwrap();
                    buf[(area.x + 2 * cx, area.y + cy)].set_bg(tile.into());
                    buf[(area.x + 2 * cx + 1, area.y + cy)].set_bg(tile.into());
                }
            }
        } else {
            for cy in 0..area.height {
                for cx in 0..area.width {
                    buf[(area.x + cx, area.y + cy)].set_bg(ratatui::style::Color::Red);
                }
            }
        }
    }
}
