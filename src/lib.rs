#[cfg(feature = "parallel")]
extern crate rayon;

pub mod drivers;
pub mod rulesets;

pub trait RuleSet {
    /// Der diesem RuleSet zugrunde liegende Zellentyp
    type Cell : Default + Copy + PartialEq;

    /// Die Regel, die angibt, wie sich die Zellen in diesem RuleSet verhalten.
    /// Das übergebene Array gibt den Zustand einer Zelle und ihrer acht Moore-Nachbarn
    /// (https://de.wikipedia.org/wiki/Moore-Nachbarschaft) wie folgt an:
    /// [[TL, T, TR].
    ///  [ L, M,  R],
    ///  [BL, B, BR]]    (Sprich: row-major-order)
    ///
    /// Der Rückgabewert ist der neue Wert für die mittlere Zelle
    fn step(neighborhood: [[<Self as RuleSet>::Cell; 3]; 3]) -> <Self as RuleSet>::Cell;
}

pub trait CellWorld<R: RuleSet> {
    /// Leg ein neues Gitter mit der angegebenen Höhe und Breite an.
    /// Alle Zellen werden mit Default::default() initialisiert.
    fn new(width: usize, height: usize) -> Self;

    /// Setz den Wert der Zelle an der angegebenen Position auf `value`
    /// Bei Koordinaten außerhalb des Gitters: beliebiges, safes Verhalten (z.b. panic, no-op)
    fn set_cell(&mut self, x: usize, y: usize, value: R::Cell);

    /// Gib der Wert der Zelle an der angegebenen Position aus.
    /// Bei Koordinaten außerhalb des Gitters: beliebiges, safes Verhalten (z.b. panic, beliebiger return value)
    fn get_cell(&self, x: usize, y: usize) -> R::Cell;

    /// Wende das Ruleset einmal auf das ganze Gitter an.
    fn step(&mut self);

    /// Wende das Ruleset `n`-mal auf das ganze Gitter an.
    /// Falls dir keine tollen Optimisationen einfallen, gibt es eine simple default-Implementation
    fn step_many(&mut self, n: usize) {
        for _ in 0..n {
            self.step();
        }
    }
}