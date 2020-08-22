use crate::{
    candidates  :: { CandidateSet, traits :: CandidatesRead },
    bitsets     :: traits::BitSet,
    indices,
};
use self::{ value_key::*, traits::* };

pub type LocationSet = std::collections::HashSet<usize>;

pub mod traits {
    use super::*;

    pub trait ValueBase: ValuesRead + ValuesModify + CandidatesRead { }

    pub trait ValuesRead {
        fn get(&self, key: &ValueKey) -> Option<&LocationSet>;
    }

    pub trait ValuesModify {
        fn insert(&mut self, key: ValueKey, values: LocationSet) -> Option<LocationSet>;
        fn insert_into(&mut self, key: ValueKey, value: usize) -> Option<&LocationSet>;

        fn remove(&mut self, key: &ValueKey) -> Option<LocationSet>;
        fn remove_from(&mut self, key: &ValueKey, value: usize) -> Option<&LocationSet>;
    }

    pub trait Values {
        fn init_values(&mut self);
        fn update_values(&mut self, index: usize, new_value: u8, old_value: u8);
    }
}

impl<T> Values for T where T: ValueBase {
    fn init_values(&mut self) {
        for i in 0..9 {
            populate(self, SetType::Row, i);
            populate(self, SetType::Col, i);
            populate(self, SetType::Box, i);
        }
    }

    fn update_values(&mut self, index: usize, new_value: u8, old_value: u8) {
        if new_value == 0 {
            // For each allowed candidate at this index insert the index to the corresponding value map
            insert_all(self, index);

            // The remaining cells in this row, column, and box can now potentially contain the value that is being
            // removed. But this depends on the other values in the rows, columns, and boxes that the other cells are
            // associated with. Never the less, we must handle this:
            for &key in rcb_value_key(index, old_value).iter() {
                insert_for(self, key);
            }
        }
        else {
            // Since every index is associated with three value records (Row, Column and Box), get the appropriate keys 
            // and perform updates for each of them
            for &key in rcb_value_key(index, new_value).iter() {
                remove_for(self, key);
            }

            // All the remaining values stored at the given index should be removed from the Row, Column and Box records
            // that contain this index. This is done as the last step because the list of locations that were stored 
            // within the RCB records for this value were needed to clean up associated cells. Since that is now done, 
            // the remaining data can be removed without exceptions.
            remove_all(self, index);
        }
    }
}

fn populate<T: ValueBase>(context: &mut T, set_type: SetType, set_index: usize) {
    let set_indices = match set_type {
        SetType::Row => indices::row_at(set_index),
        SetType::Col => indices::col_at(set_index),
        SetType::Box => indices::box_at(set_index)
    };

    for &index in set_indices.iter() {
        let candidate_set = match CandidatesRead::get(context, index) {
            Some(set) => *set,
            None => continue
        };

        for candidate in candidate_set.iter() {
            context.insert_into(ValueKey { value: candidate.into(), set_type, set_index }, index);
        }
    }
}

fn insert_for<T: ValueBase>(context: &mut T, key: ValueKey) {
    let affected = match key.set_type {
        SetType::Row => (indices::row_at(key.set_index), [SetType::Col, SetType::Box]),
        SetType::Col => (indices::col_at(key.set_index), [SetType::Row, SetType::Box]),
        SetType::Box => (indices::box_at(key.set_index), [SetType::Row, SetType::Col]),
    };

    for &index in affected.0.iter() {
        let candidate_set = match CandidatesRead::get(context, index) {
            Some(set) => *set,
            None => continue
        };

        if !candidate_set.contains(CandidateSet::from(key.value)) {
            continue;
        }

        context.insert_into(key, index);

        for &set_type in affected.1.iter() {
            let associated_key = ValueKey {
                value: key.value,
                set_type,
                set_index:  match set_type {
                    SetType::Row => indices::row_index(index),
                    SetType::Col => indices::col_index(index),
                    SetType::Box => indices::box_index(index)
                }
            };
            context.insert_into(associated_key, index);
        }
    }
}

fn remove_for<T: ValueBase>(context: &mut T, key: ValueKey) {
    // Start by removing the value, along with all of its locations, from within whichever collection the given key
    // refers to
    let location_set = match context.remove(&key) {
        Some(set)   => set,
        None        => return
    };

    // Next, for each of the locations where the value could have been present within this collection, we need to
    // update the other two types of collections and remove those specific indices.
    let associated_sets = match key.set_type {
        SetType::Row => [SetType::Col, SetType::Box],
        SetType::Col => [SetType::Row, SetType::Box],
        SetType::Box => [SetType::Row, SetType::Col]
    };

    for &cell_index in location_set.iter() {
        for &set_type in associated_sets.iter() {
            let key = ValueKey {
                value: key.value,
                set_type, 
                set_index:  match set_type {
                    SetType::Row => indices::row_index(cell_index),
                    SetType::Col => indices::col_index(cell_index),
                    SetType::Box => indices::box_index(cell_index)
                }
            };

            match context.remove_from(&key, cell_index) {
                Some(remaining) => if remaining.is_empty() { context.remove(&key); },
                None => continue
            }
        }
    }
}

fn insert_all<T: ValueBase>(context: &mut T, index: usize) {
    // For each possible candidate at this index insert the index to the corresponding value map
    let candidate_set = match CandidatesRead::get(context, index) {
        Some(set) => *set,
        None => return
    };

    for candidate in candidate_set.iter() {
        for key in rcb_value_key(index, candidate.into()).iter() {
            context.insert_into(*key, index);
        }
    }
}

// Given an index and a value, removes all values at the index from the value map
fn remove_all<T: ValueBase>(context: &mut T, index: usize) {
    let candidate_set = match CandidatesRead::get(context, index) {
        Some(set) => *set,
        None => return
    };

    for candidate in candidate_set.iter() {
        for key in rcb_value_key(index, candidate.into()).iter() {
            match context.remove_from(key, index) {
                Some(remaining) => if remaining.is_empty() { context.remove(key); },
                None => continue
            }
        }
    }
}

#[inline]
fn rcb_value_key(cell_index: usize, value: u8) -> [ValueKey; 3] {[ 
    ValueKey { value, set_type: SetType::Row, set_index: indices::row_index(cell_index) }, 
    ValueKey { value, set_type: SetType::Col, set_index: indices::col_index(cell_index) }, 
    ValueKey { value, set_type: SetType::Box, set_index: indices::box_index(cell_index) }
]}

pub mod value_key {
    #[repr(u8)]
    #[derive(PartialEq, Eq, Ord, PartialOrd, Hash, Copy, Clone, Debug)]
    pub enum SetType { Row, Col, Box }

    impl ::core::fmt::Display for SetType {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.write_str(
                match &self { 
                    SetType::Row => "R",
                    SetType::Col => "C",
                    SetType::Box => "B"
                }
            )
        }
    }

    #[derive(PartialEq, Eq, Ord, PartialOrd, Hash, Copy, Clone, Debug)]
    pub struct ValueKey {
        pub value:      u8,
        pub set_type:   SetType,
        pub set_index:  usize
    }

    impl ::core::fmt::Display for ValueKey {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.write_fmt(format_args!("{} ({:?}:{:?})", self.value, self.set_type, self.set_index))
        }
    }
}