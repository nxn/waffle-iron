use crate::{
    sudoku  :: traits :: SudokuState,
    bitsets :: traits :: BitSetBase,
    indices,
};

use self::traits::*;

bitflags! {
    pub struct CandidateSet: u16 {
        const C1 = 0x0001;
        const C2 = 0x0002;
        const C3 = 0x0004;
        const C4 = 0x0008;
        const C5 = 0x0010;
        const C6 = 0x0020;
        const C7 = 0x0040;
        const C8 = 0x0080;
        const C9 = 0x0100;
    }
}

impl Default for CandidateSet {
    fn default() -> Self {
        CandidateSet::empty()
    }
}

impl From<u8> for CandidateSet {
    fn from(item: u8) -> CandidateSet {
        match item {
            1 => CandidateSet::C1,
            2 => CandidateSet::C2,
            3 => CandidateSet::C3,
            4 => CandidateSet::C4,
            5 => CandidateSet::C5,
            6 => CandidateSet::C6,
            7 => CandidateSet::C7,
            8 => CandidateSet::C8,
            9 => CandidateSet::C9,
            _ => CandidateSet::empty()
        }
    }
}

impl Into<u8> for CandidateSet {
    fn into(self) -> u8 {
        match self {
            CandidateSet::C1 => 1,
            CandidateSet::C2 => 2,
            CandidateSet::C3 => 3,
            CandidateSet::C4 => 4,
            CandidateSet::C5 => 5,
            CandidateSet::C6 => 6,
            CandidateSet::C7 => 7,
            CandidateSet::C8 => 8,
            CandidateSet::C9 => 9,
            _ => 0
        }
    }
}

impl BitSetBase<u16, CandidateSet> for CandidateSet { 
    #[inline]
    fn bits(&self) -> u16 {
        self.bits()
    }

    #[inline]
    fn from_bits(bits: u16) -> Option<CandidateSet> {
        CandidateSet::from_bits(bits)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

pub mod traits {
    use super::*;

    pub trait CandidateBase: SudokuState + CandidatesRead + CandidatesModify { }

    pub trait CandidatesRead {
        fn get(&self, index: usize) -> Option<&CandidateSet>;
    }

    pub trait CandidatesModify {
        fn insert(&mut self, index: usize, candidates: CandidateSet) -> Option<CandidateSet>;
        fn remove(&mut self, index: usize) -> Option<CandidateSet>;
        fn remove_diff(&mut self, index: usize, diff: CandidateSet) -> Option<&CandidateSet>;
    }
    
    pub trait Candidates {
        fn init_candidates(&mut self);
        fn update_candidates(&mut self, index: usize, new_value: u8, old_value: u8);
    }
}

impl<T> Candidates for T where T: CandidateBase {
    fn init_candidates(&mut self) {
        generate_candidates(self, 0..81);
    }

    fn update_candidates(&mut self, index: usize, new_value: u8, _old_value: u8) {
        if new_value == 0 {
            // TODO: Make the previous value available and optimize this instead of regenerating from scratch? Might not
            // be worth the effort.
            generate_candidates(self, indices::rcb_containing(index).iter().copied());
        }
        else {
            remove_candidates(self, index, new_value);
        }
    }
}

fn generate_candidates<T: CandidateBase>(context: &mut T, indices: impl IntoIterator<Item=usize>) {
    for cell_index in indices {
        // If cell already has value no candidates are needed
        if let Some(value) = SudokuState::get(context, cell_index) {
            if value > 0 { continue; }
        }

        // For each cell associated with this one (ie: by being contained within this cell's row, column, or box 
        // group), remove this cell's value from their candidate sets.
        let mut candidates = CandidateSet::all();
        for &associated_index in indices::rcb_containing(cell_index) {
            if let Some(value) = SudokuState::get(context, associated_index) {
                candidates.remove(CandidateSet::from(value))
            };
        }

        context.insert(cell_index, candidates);
    }
}

fn remove_candidates<T: CandidateBase>(context: &mut T, index: usize, value: u8) {
    context.remove(index);

    for cell_index in indices::rcb_containing(index) {
        match context.remove_diff(*cell_index, CandidateSet::from(value)) {
            Some(remaining) => 
                if remaining.is_empty() {
                    context.remove(*cell_index);
                },
            None => continue
        }
    }
}