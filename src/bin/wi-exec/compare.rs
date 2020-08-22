use std :: {
    hash :: { Hash, BuildHasher },
    cmp  :: { Eq, PartialEq },
    fmt  :: { Display, Debug }
};

use self::traits::*;

use im_rc :: { 
    HashMap,
    hashmap::Iter
};

#[derive(PartialEq)]
pub enum Comparison { Ptr, Val }

pub mod traits {
    use super::*;

    pub trait ReadCollection<'a, K:'a, V:'a> {
        type Iter: Iterator<Item = &'a (K, V)>;
    
        fn iter(&'a self) -> Self::Iter;
        fn get(&self, key: &K) -> Option<&V>;
    }
    
    pub trait Compare<'a, T, K:'a, V:'a> {
        fn compare(&'a self, other: &'a T, compare: Comparison);
    }
}

impl<'a, K:'a, V:'a, H> ReadCollection<'a, K, V> for HashMap<K, V, H>
where   K: Hash + Eq,
        H: BuildHasher {
    type Iter = Iter<'a, K, V>;

    fn iter(&self) -> Iter<'_, K, V> {
        self.iter()
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.get(key)
    }
}

impl<'a, T, K:'a, V: 'a> traits::Compare<'a, T, K, V> for T 
where   T: ReadCollection<'a, K, V>,
        K: Display + PartialEq, 
        V: Debug   + PartialEq + Default {
    fn compare(&'a self, other: &'a T, cmp: Comparison) {
        compare(self, other, cmp);
    }
}

pub fn compare<'a, K:'a, V:'a, T>(old_collection: &'a T, new_collection: &'a T, compare: Comparison)
where   T: ReadCollection<'a, K, V>,
        K: Display + PartialEq, 
        V: Debug   + PartialEq + Default {

    let mut ptr_diff_count = 0;
    let mut val_diff_count = 0;
    let mut total = 0;

    for (key, old) in old_collection.iter() {
        let empty = V::default();
        let new = match new_collection.get(&key) {
            Some(thing) => thing,
            None        => &empty
        };

        total += 1;

        if old != new {
            val_diff_count += 1;
            if compare == Comparison::Val {
                println!("{:>9} :: {:>32} -> {:?}", key, format!("{:?}", old),  new);
            }
        }

        // Only increment ptr_diff_count if values are the same but ptrs don't match
        else if !std::ptr::eq(old, new) {
            ptr_diff_count += 1;
            if compare == Comparison::Ptr {
                println!("{:>9} :: {:p} -> {:p}", key, &old, new);
            }
        }
    }

    println!("Changes (val / ptr / total): {} / {} / {}", val_diff_count, ptr_diff_count, total)
}
