use crate :: {
    candidates  :: { CandidateSet, traits :: * },
    values      :: { LocationSet, value_key :: ValueKey, traits :: * },
    bitsets     :: traits :: BitSet,
};

use self :: {
    choices :: *, traits :: *
};

use std :: {
    collections :: hash_map :: { DefaultHasher, RandomState },
    hash        :: BuildHasherDefault
};

use im_rc :: { Vector, HashMap, OrdSet };

type _StaticHasher = BuildHasherDefault<DefaultHasher>;
type RandomHasher = RandomState;

type Puzzle         = im_rc::Vector<u8>;
type PuzzleIter<'a> = im_rc::vector::Iter<'a, u8>;
type CandidateMap   = im_rc::HashMap<usize, CandidateSet, RandomHasher>;
type ValueMap       = im_rc::HashMap<ValueKey, LocationSet, RandomHasher>;
type ChoiceQueue    = im_rc::OrdSet<Choice>;

pub mod traits {
    use super::PuzzleIter;

    pub trait SudokuState: Clone {
        fn new(puzzle: &[u8; 81]) -> Self;
        fn next_choice(&self) -> Vec<(usize, u8)>;
        fn get(&self, index: usize) -> Option<u8>;
        fn set(&self, index: usize, value: u8) -> Self;
        fn iter(&self) -> PuzzleIter;
        fn remaining(&self) -> usize;
    }
}

pub struct Sudoku {
    puzzle: Puzzle,
    remaining: usize,

    // Stores set of all allowable candidate values at each non-empty cell index. This is calculated based on existing
    // values of the Row, Column, and Box sets of each individual cell.
    candidate_map: CandidateMap,

    // The collection of candidate sets above is actually enough data to solve any valid sudoku puzzle. However, to
    // optimize the efficiency of the solving algorithm we need to keep track of a few additional data structures to
    // help reduce unnecessary branching while solving.
    //
    // Namely, for each row, column, and box, store missing values mapped to their possible locations within this set.
    // This helps identify cases where a value can only be placed into a single position within a row, column, or box.
    value_map: ValueMap,

    // Keys of both of the above structures are also placed within a container that identifies the number of choices
    // held by the key. This is stored within an ordered queue so that we can prioritize placing values for cells that
    // have a lower number of valid values/candidates.
    choices: ChoiceQueue,
}

impl Sudoku {
    fn update(&mut self, index: usize, new_value: u8) {
        let old_value = self.puzzle[index];

        if old_value == new_value { return; }

        if old_value == 0 && new_value > 0 {
            self.remaining -= 1;
        }
        else if old_value > 0 && new_value == 0 {
            self.remaining += 1;
        }

        self.puzzle[index] = new_value;

        // TODO: This is an ugly hack necessary because the value map is depended on the state of the candidate map. 
        // Depending on whether a value is being added or removed the value_map expects the candidate map to either have
        // or not have the new state. Not sure how to clean this uglyness up yet.
        if new_value == 0 {
            self.update_candidates(index, new_value, old_value);
            self.update_values(index, new_value, old_value);
        } else {
            self.update_values(index, new_value, old_value);
            self.update_candidates(index, new_value, old_value);
        }
    }
}

impl SudokuState for Sudoku {
    fn new(puzzle: &[u8; 81]) -> Self {
        let mut yaws = Sudoku {
            puzzle:         Vector::from(&puzzle[..]),
            remaining:      puzzle.iter().fold(0, |c, &v| if v == 0 { c + 1 } else { c }),

            candidate_map:  HashMap::default(),
            value_map:      HashMap::default(),
            choices:        OrdSet::default(),
        };

        yaws.init_candidates();
        yaws.init_values();

        yaws
    }

    fn next_choice(&self) -> Vec<(usize, u8)> {
        let queue_item = match self.choices.get_min() {
            Some(item) => item,
            None => return Vec::default()
        };

        // TODO: This can probably be cleaned up a bit
        match queue_item.key {
            CollectionKey::Candidates(key) =>
                self.candidate_map.get(&key).unwrap().iter().map(|set| (key, set.into())).collect(),

            CollectionKey::Values(key) =>
                self.value_map.get(&key).unwrap().iter().map(|&index| (index, key.value)).collect()
        }
    }

    #[inline]
    fn get(&self, index: usize) -> Option<u8> {
        self.puzzle.get(index).copied()
    }

    // Creates a clone of the current state and applies the given change to the new copy.
    #[inline]
    fn set(&self, index: usize, value: u8) -> Self {
        let mut clone = self.clone();
        clone.update(index, value);
        clone
    }

    #[inline]
    fn iter(&self) -> PuzzleIter {
        self.puzzle.iter()
    }

    #[inline]
    fn remaining(&self) -> usize {
        self.remaining
    }
}

impl IntoIterator for Sudoku {
    type Item = u8;
    type IntoIter = im_rc::vector::ConsumingIter<u8>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.puzzle.into_iter()
    }
}

impl<'a> IntoIterator for &'a Sudoku {
    type Item = &'a u8;
    type IntoIter =  im_rc::vector::Iter<'a, u8>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.puzzle.iter()
    }
}

impl From<[u8; 81]> for Sudoku {
    fn from(array: [u8; 81]) -> Sudoku {
        Sudoku::new(&array)
    }
}

impl Into<[u8; 81]> for Sudoku {
    fn into(self) -> [u8; 81] {
        let mut array = [0; 81];
        for (i, cell_value) in self.into_iter().enumerate() {
            array[i] = cell_value;
        }
        array
    }
}

impl Clone for Sudoku {
    // Since the struct is composed of immutable data structures the clone operations do not immediately clone anything.
    // Actual copies are only made as needed once changes start being written to either structure. The end result should
    // be a data structure that shares references to its previous state if no modifications were made to those specific
    // parts of the data.
    fn clone(&self) -> Self {
        Sudoku {
            puzzle:         self.puzzle.clone(),
            remaining:      self.remaining,

            candidate_map:  self.candidate_map.clone(),
            value_map:      self.value_map.clone(),
            choices:        self.choices.clone(),
        }
    }
}

#[macro_use]
mod impl_macros {
    // The insert and remove logic is basically identical between the candidate_map and value_map, however, to the best
    // of my knowledge, there does not seem to be a practical way of creating a generic implementation for both. This is
    // due to the fact that an additional mutable reference would have to be passed in to designate which collection
    // is going to be modified. However, doing so is not allowed as we already hold a mutable reference to "self" which
    // owns both collections. While there are approaches for dealing with this scenario, they typically relly on runtime
    // borrow checking, or would necessitate boxing of the collections to work around sizing differences.

    // Both options are overkill in terms of simply wanting to eliminate a bit of code duplication, so instead, macros
    // are used to generate the necessary operations.
    macro_rules! impl_insert {
        ($self:ident, $collection:ident, $key:expr, $val:expr, $collection_key:expr) => {{
            let new_len = $val.len();
            let out = $self.$collection.insert($key, $val);

            // If the insert operation overwrote an existing record, ensure the corresponding record is removed from the
            // solve_queue if the number of options are now different. Note: the actual values are not relevant, since
            // the choice queue only stores the number of possibilities plus a key to whichever collection stores the
            // values.
            if let Some(set) = &out {
                let old_len = set.len();
                if old_len != new_len {
                    $self.choices.remove( &Choice::new(old_len, $collection_key) );
                }
            }

            $self.choices.insert( Choice::new(new_len, $collection_key) );

            out
        }}
    }

    macro_rules! impl_remove {
        ($self:ident, $collection:ident, $key:expr, $collection_key:expr) => {{
            let out = $self.$collection.remove($key);

            if let Some(set) = &out {
                $self.choices.remove( &Choice::new(set.len(), $collection_key) );
            }

            out
        }}
    }

    macro_rules! impl_insert_into {
        ($self:ident, $collection:ident, $key:expr, $val:expr, $new:expr, $collection_key:expr) => {{
            let set = $self.$collection.entry($key).or_insert_with($new);

            let old_len = set.len();
            set.insert($val);
            let new_len = set.len();

            if old_len != new_len {
                $self.choices.remove( &Choice::new(old_len, $collection_key) );
                $self.choices.insert( Choice::new(new_len, $collection_key) );
            }

            Some(set)
        }}
    }

    macro_rules! impl_remove_from {
        ($self:ident, $collection:ident, $key:expr, $val:expr, $collection_key:expr) => {{
            let set = match $self.$collection.get_mut($key) {
                Some(set) => set,
                None => return None
            };

            let old_len = set.len();
            set.remove($val);
            let new_len = set.len();

            if old_len != new_len {
                $self.choices.remove( &Choice::new(old_len, $collection_key) );
                $self.choices.insert( Choice::new(new_len, $collection_key) );
            }

            Some(set)
        }}
    }
}

impl CandidateBase for Sudoku { }
impl ValueBase     for Sudoku { }

impl CandidatesRead for Sudoku {
    fn get(&self, index: usize) -> Option<&CandidateSet> {
        self.candidate_map.get(&index)
    }
}

impl CandidatesModify for Sudoku {
    fn insert(&mut self, index: usize, candidates: CandidateSet) -> Option<CandidateSet> {
        impl_insert!(self, candidate_map, index, candidates, CollectionKey::Candidates(index))
    }

    fn remove(&mut self, index: usize) -> Option<CandidateSet> {
        impl_remove!(self, candidate_map, &index, CollectionKey::Candidates(index))
    }

    fn remove_diff(&mut self, index: usize, diff: CandidateSet) -> Option<&CandidateSet> {
        impl_remove_from!(self, candidate_map, &index, diff, CollectionKey::Candidates(index))
    }
}

impl ValuesRead for Sudoku {
    fn get(&self, key: &ValueKey) -> Option<&LocationSet> {
        self.value_map.get(key)
    }
}

impl ValuesModify for Sudoku {
    fn insert(&mut self, key: ValueKey, values: LocationSet) -> Option<LocationSet> {
        impl_insert!(self, value_map, key, values, CollectionKey::Values(key))
    }

    fn insert_into(&mut self, key: ValueKey, value: usize) -> Option<&LocationSet> {
        impl_insert_into!(self, value_map, key, value, LocationSet::default, CollectionKey::Values(key))
    }

    fn remove(&mut self, key: &ValueKey) -> Option<LocationSet> {
        impl_remove!(self, value_map, key, CollectionKey::Values(*key))
    }

    fn remove_from(&mut self, key: &ValueKey, index: usize) -> Option<&LocationSet> {
        impl_remove_from!(self, value_map, key, &index, CollectionKey::Values(*key))
    }
}

mod choices {
    use super::ValueKey;

    #[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Debug)]
    pub enum CollectionKey {
        Candidates(usize),
        Values(ValueKey),
    }

    #[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
    pub struct Choice {
        pub possibilities: usize,
        pub key: CollectionKey
    }

    impl Choice {
        pub fn new(possibilities: usize, key: CollectionKey) -> Choice {
            Choice { possibilities, key }
        }
    }
}