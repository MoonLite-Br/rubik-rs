use super::lessons::LESSONS;
use crate::cube::{Cube, Move};

/// Step state machine for one tutorial case: `step` = number of moves already applied.
/// The caller animates the returned move and applies it to the displayed cube.
pub struct Player {
    pub lesson: usize,
    pub case: usize,
    pub moves: Vec<Move>,
    pub step: usize,
}

impl Player {
    pub fn new(lesson: usize, case: usize) -> Player {
        let c = &LESSONS[lesson].cases[case];
        Player { lesson, case, moves: c.moves(), step: 0 }
    }

    /// Cube state before the first move.
    pub fn start(&self) -> Cube {
        LESSONS[self.lesson].cases[self.case].start()
    }

    pub fn at_end(&self) -> bool {
        self.step >= self.moves.len()
    }

    /// Next move of the algorithm, if any.
    pub fn next(&mut self) -> Option<Move> {
        if self.step < self.moves.len() {
            let m = self.moves[self.step];
            self.step += 1;
            Some(m)
        } else {
            None
        }
    }

    /// Undo one step: returns the inverse of the last applied move.
    pub fn prev(&mut self) -> Option<Move> {
        if self.step > 0 {
            self.step -= 1;
            Some(self.moves[self.step].inverse())
        } else {
            None
        }
    }
}
