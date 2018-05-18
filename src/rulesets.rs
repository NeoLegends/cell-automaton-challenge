use ::RuleSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryCell {
    Live,
    Dead,
}

impl Default for BinaryCell {
    fn default() -> Self { BinaryCell::Dead }
}

// das altbekannte Conway's Game of Life
#[derive(Debug)]
pub struct GameOfLife;

impl RuleSet for GameOfLife {
    type Cell = self::BinaryCell;

    fn step([[tl, t, tr],
                [l,  m, r ],
                [bl, b, br]]: [[BinaryCell; 3]; 3]) -> BinaryCell {
        use self::BinaryCell::*;
        let live_neighbors = [tl, t, tr, l, r, bl, b, br].iter().filter(|&&x| x == Live).count();
        match (m, live_neighbors) {
            (_, 3)    => Live,
            (Live, 2) => Live,
            _         => Dead,
        }
    }
}

// sehr simple Simulation einer Diffusion
#[derive(Debug)]
pub struct Diffusion;

impl RuleSet for Diffusion {
    type Cell = f32;

    fn step([[tl, t, tr],
                [l,  m, r ],
                [bl, b, br]]: [[f32; 3]; 3]) -> f32 {
        0.05*tl + 0.1*t + 0.05*tr +
        0.1 * l + 0.4*m + 0.1 * r +
        0.05*bl + 0.1*b + 0.05*br
    }
}