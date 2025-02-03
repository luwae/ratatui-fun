use std::fmt;

use rand::{seq::SliceRandom, Rng, RngCore};

#[derive(Debug)]
struct UnionFind {
    reps: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            reps: Vec::from_iter(0..n),
        }
    }

    fn rep(&self, mut a: usize) -> usize {
        while self.reps[a] != a {
            a = self.reps[a];
        }
        a
    }

    fn in_same_set(&self, a: usize, b: usize) -> bool {
        self.rep(a) == self.rep(b)
    }

    fn join(&mut self, a: usize, b: usize) {
        let rep_b = self.rep(b);
        self.reps[rep_b] = a;
    }
}

impl fmt::Display for UnionFind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UnionFind {{ ")?;
        for rep in 0..self.reps.len() {
            if self.reps[rep] != rep {
                continue;
            }
            // find everything belonging to this rep
            write!(f, "[")?;
            let mut first = true;
            for member in 0..self.reps.len() {
                if self.in_same_set(member, rep) {
                    if first {
                        first = false;
                        write!(f, "{}({})", member, self.reps[member])?;
                    } else {
                        write!(f, ", {}({})", member, self.reps[member])?;
                    }
                }
            }
            write!(f, "] ")?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

#[derive(Copy, Clone)]
pub enum Tile {
    Free,
    Wall,
}

pub struct Maze {
    pub tiles: Vec<Vec<Tile>>,
}

#[derive(Copy, Clone, Debug)]
struct Pos(usize, usize);

fn is_horizontal_edge(pos: Pos) -> bool {
    pos.0 % 2 == 0
}

fn node_to_idx(pos: Pos, nx: usize, ny: usize) -> usize {
    (pos.1 / 2) * nx + (pos.0) / 2
}

impl Maze {
    pub fn empty(nx: usize, ny: usize) -> Self {
        let mut maze = Maze { tiles: Vec::new() };
        for line in 0..(2 * ny + 1) {
            if line % 2 == 0 {
                maze.tiles.push(vec![Tile::Wall; 2 * nx + 1]);
            } else {
                let mut ln = Vec::new();
                for _ in 0..nx {
                    ln.push(Tile::Wall);
                    ln.push(Tile::Free);
                }
                ln.push(Tile::Wall);
                maze.tiles.push(ln);
            }
        }
        maze
    }

    pub fn kruskal(nx: usize, ny: usize) -> Self {
        let mut maze = Self::empty(nx, ny);
        let mut edges = Vec::new();
        // horizontal
        for y in 0..ny {
            for x in 1..nx {
                edges.push(Pos(2 * x, 2 * y + 1));
            }
        }
        // vertical
        for x in 0..nx {
            for y in 1..ny {
                edges.push(Pos(2 * x + 1, 2 * y));
            }
        }
        edges.shuffle(&mut rand::rng());

        let mut sets = UnionFind::new(nx * ny);
        let mut unused_edges = Vec::new();

        for edge in edges {
            let (pos_a, pos_b) = if is_horizontal_edge(edge) {
                (Pos(edge.0 - 1, edge.1), Pos(edge.0 + 1, edge.1))
            } else {
                (Pos(edge.0, edge.1 - 1), Pos(edge.0, edge.1 + 1))
            };
            let (index_a, index_b) = (node_to_idx(pos_a, nx, ny), node_to_idx(pos_b, nx, ny));
            if !sets.in_same_set(index_a, index_b) {
                sets.join(index_a, index_b);
                maze.tiles[edge.1][edge.0] = Tile::Free;
            } else {
                unused_edges.push(edge);
            }
        }

        // remove some random edges taht are still standing
        unused_edges.shuffle(&mut rand::rng());
        let n = (nx * ny) / 2;
        for edge in &unused_edges[..n] {
            maze.tiles[edge.1][edge.0] = Tile::Free;
        }
        /*
        // remove random Wall tiles
        let mut to_remove = (nx * ny) / 1;
        while to_remove > 0 {
            let x = rand::rng().random_range(1..(nx * 2));
            let y = rand::rng().random_range(1..(ny * 2));
            if let Tile::Wall = maze.tiles[y][x] {
                maze.tiles[y][x] = Tile::Free;
                to_remove -= 1;
            }
        }
        */

        maze
    }
}

impl fmt::Display for Maze {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in &self.tiles {
            for tile in line {
                match tile {
                    Tile::Free => write!(f, " ")?,
                    Tile::Wall => write!(f, "O")?,
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
