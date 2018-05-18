use std::fmt::{self, Display, Formatter};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use ::{CellWorld, RuleSet};

/// Gibt potentielle eine Adjazenz zum Rand der Matrix an.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Adjacency { // Bitflags vllt. besser hier?
    Top,
    Right,
    Bottom,
    Left,
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,

    None,
}

/// Eine Gitter-Engine, welche Werte am Rand mit Default::default() emuliert.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Driver<R: RuleSet> {
    data: Vec<R::Cell>,
    width: usize,
}

impl<R: RuleSet> Driver<R> {
    /// Erstelle neuen Driver mit geg. Breite und Höhe.
    fn new_with(width: usize, height: usize) -> Self {
        Self {
            data: (0..(width * height)).map(|_| Default::default()).collect(),
            width,
        }
    }

    /// Berechnet, ob die Zelle am gegebenen Index adjazent zu einem der Ränder ist.
    fn adjacency(&self, idx: usize) -> Adjacency {
        let x = idx % self.width;
        let y = idx / self.width;

        if y == 0 {
            if x == 0 {
                Adjacency::TopLeft
            } else if x == self.width - 1 {
                Adjacency::TopRight
            } else {
                Adjacency::Top
            }
        } else if y == (self.data.len() / self.width) - 1 {
            if x == 0 {
                Adjacency::BottomLeft
            } else if x == self.width - 1 {
                Adjacency::BottomRight
            } else {
                Adjacency::Bottom
            }
        } else {
            if x == 0 {
                Adjacency::Left
            } else if x == self.width - 1 {
                Adjacency::Right
            } else {
                Adjacency::None
            }
        }
    }

    /// Rufe Wert einer Zelle ab.
    fn get(&self, x: usize, y: usize) -> R::Cell {
        self.data[y * self.width + x]
    }

    /// Holt die 3x3-Matrix um die Zelle mit geg. Index.
    fn get_field_matrix(&self, idx: usize) -> [[R::Cell; 3]; 3] {
        let adj = self.adjacency(idx);
        let tl = match adj {
            Adjacency::Top | Adjacency::TopLeft | Adjacency::TopRight | Adjacency::Left => Default::default(),
            _ => self.data[idx - self.width - 1],
        };
        let t = match adj {
            Adjacency::Top | Adjacency::TopLeft | Adjacency::TopRight => Default::default(),
            _ => self.data[idx - self.width],
        };
        let tr = match adj {
            Adjacency::Top | Adjacency::TopLeft | Adjacency::TopRight | Adjacency::Right => Default::default(),
            _ => self.data[idx - self.width + 1],
        };
        let l = match adj {
            Adjacency::TopLeft | Adjacency::BottomLeft | Adjacency::Left => Default::default(),
            _ => self.data[idx - 1],
        };
        let c = self.data[idx];
        let r = match adj {
            Adjacency::TopRight | Adjacency::BottomRight | Adjacency::Right => Default::default(),
            _ => self.data[idx + 1],
        };
        let bl = match adj {
            Adjacency::Bottom | Adjacency::BottomLeft | Adjacency::BottomRight | Adjacency::Left => Default::default(),
            _ => self.data[idx + self.width - 1],
        };
        let b = match adj {
            Adjacency::Bottom | Adjacency::BottomLeft | Adjacency::BottomRight => Default::default(),
            _ => self.data[idx + self.width],
        };
        let br = match adj {
            Adjacency::Bottom | Adjacency::BottomLeft | Adjacency::BottomRight | Adjacency::Right => Default::default(),
            _ => self.data[idx + self.width + 1],
        };

        [
            [tl, t, tr],
            [l, c, r],
            [bl, b, br],
        ]
    }

    /// Setze Wert einer Zelle.
    fn set(&mut self, x: usize, y: usize, value: R::Cell) {
        self.data[y * self.width + x] = value;
    }
}

#[cfg(not(feature = "parallel"))]
impl<R: RuleSet> CellWorld<R> for Driver<R> {
    /// Leg ein neues Gitter mit der angegebenen Höhe und Breite an.
    /// Alle Zellen werden mit Default::default() initialisiert.
    fn new(width: usize, height: usize) -> Self {
        Self::new_with(width, height)
    }

    /// Setz den Wert der Zelle an der angegebenen Position auf `value`
    /// Bei Koordinaten außerhalb des Gitters: beliebiges, safes Verhalten (z.b. panic, no-op)
    fn set_cell(&mut self, x: usize, y: usize, value: R::Cell) {
        self.set(x, y, value)
    }

    /// Gib der Wert der Zelle an der angegebenen Position aus.
    /// Bei Koordinaten außerhalb des Gitters: beliebiges, safes Verhalten (z.b. panic, beliebiger return value)
    fn get_cell(&self, x: usize, y: usize) -> R::Cell {
        self.get(x, y)
    }

    /// Wende das Ruleset einmal auf das ganze Gitter an.
    fn step(&mut self) {
        self.data = (0..self.data.len())
            .map(|idx| R::step(self.get_field_matrix(idx)))
            .collect();
    }
}

#[cfg(feature = "parallel")]
impl<R: RuleSet> CellWorld<R> for Driver<R>
        where R::Cell: Send + Sync {
    /// Leg ein neues Gitter mit der angegebenen Höhe und Breite an.
    /// Alle Zellen werden mit Default::default() initialisiert.
    fn new(width: usize, height: usize) -> Self {
        Self::new_with(width, height)
    }

    /// Setz den Wert der Zelle an der angegebenen Position auf `value`
    /// Bei Koordinaten außerhalb des Gitters: beliebiges, safes Verhalten (z.b. panic, no-op)
    fn set_cell(&mut self, x: usize, y: usize, value: R::Cell) {
        self.set(x, y, value)
    }

    /// Gib der Wert der Zelle an der angegebenen Position aus.
    /// Bei Koordinaten außerhalb des Gitters: beliebiges, safes Verhalten (z.b. panic, beliebiger return value)
    fn get_cell(&self, x: usize, y: usize) -> R::Cell {
        self.get(x, y)
    }

    /// Wende das Ruleset einmal auf das ganze Gitter an.
    fn step(&mut self) {
        self.data = (0..self.data.len()).into_par_iter()
            .map(|idx| R::step(self.get_field_matrix(idx)))
            .collect();
    }
}

impl<R: RuleSet> Display for Driver<R>
    where R::Cell: fmt::Debug {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        for row in 0..(self.data.len() / self.width) {
            let base = row * self.width;
            writeln!(fmt, "{:?}", &self.data[base..(base + self.width)])?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rulesets::{BinaryCell, Diffusion, GameOfLife};

    #[test]
    fn test_diffusion() {
        let mut diff: Driver<Diffusion> = Driver::new(3, 3);
        diff.set_cell(0, 0, 100f32);
        diff.step();

        assert_eq!(diff.get_cell(0, 0), 40f32);
        assert_eq!(diff.get_cell(1, 0), 10f32);
        assert_eq!(diff.get_cell(2, 0), 0f32);
        assert_eq!(diff.get_cell(0, 1), 10f32);
        assert_eq!(diff.get_cell(1, 1), 5f32);
        assert_eq!(diff.get_cell(2, 1), 0f32);
        assert_eq!(diff.get_cell(0, 2), 0f32);
        assert_eq!(diff.get_cell(1, 2), 0f32);
        assert_eq!(diff.get_cell(2, 2), 0f32);
    }

    #[test]
    fn test_gol() {
        let mut gol: Driver<GameOfLife> = Driver::new(3, 3);
        gol.set_cell(1, 0, BinaryCell::Live);
        gol.set_cell(1, 1, BinaryCell::Live);
        gol.set_cell(1, 2, BinaryCell::Live);

        for _ in 0..10 { // Test pulsating behavior
            gol.step();

            assert_eq!(gol.get_cell(0, 0), BinaryCell::Dead);
            assert_eq!(gol.get_cell(1, 0), BinaryCell::Dead);
            assert_eq!(gol.get_cell(2, 0), BinaryCell::Dead);
            assert_eq!(gol.get_cell(0, 1), BinaryCell::Live);
            assert_eq!(gol.get_cell(1, 1), BinaryCell::Live);
            assert_eq!(gol.get_cell(2, 1), BinaryCell::Live);
            assert_eq!(gol.get_cell(0, 2), BinaryCell::Dead);
            assert_eq!(gol.get_cell(1, 2), BinaryCell::Dead);
            assert_eq!(gol.get_cell(2, 2), BinaryCell::Dead);

            gol.step();

            assert_eq!(gol.get_cell(0, 0), BinaryCell::Dead);
            assert_eq!(gol.get_cell(1, 0), BinaryCell::Live);
            assert_eq!(gol.get_cell(2, 0), BinaryCell::Dead);
            assert_eq!(gol.get_cell(0, 1), BinaryCell::Dead);
            assert_eq!(gol.get_cell(1, 1), BinaryCell::Live);
            assert_eq!(gol.get_cell(2, 1), BinaryCell::Dead);
            assert_eq!(gol.get_cell(0, 2), BinaryCell::Dead);
            assert_eq!(gol.get_cell(1, 2), BinaryCell::Live);
            assert_eq!(gol.get_cell(2, 2), BinaryCell::Dead);
        }

        let mut gol2: Driver<GameOfLife> = Driver::new(2, 2);
        gol2.set_cell(0, 0, BinaryCell::Live);
        gol2.set_cell(0, 1, BinaryCell::Live);
        gol2.set_cell(1, 0, BinaryCell::Live);
        gol2.set_cell(1, 1, BinaryCell::Live);

        gol.step_many(100); // Step lots of times

        assert_eq!(gol2.get_cell(0, 0), BinaryCell::Live);
        assert_eq!(gol2.get_cell(0, 1), BinaryCell::Live);
        assert_eq!(gol2.get_cell(1, 0), BinaryCell::Live);
        assert_eq!(gol2.get_cell(1, 1), BinaryCell::Live);
    }
}