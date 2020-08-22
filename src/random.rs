use std::iter::FromIterator;

pub mod traits {
    use super::RandomIter;

    pub trait Random<'a, T> {
        fn random(self) -> RandomIter<'a, T>;
    }
}

pub struct RandomIter<'a, T> {
    items:  Vec<&'a T>,
    done:   bool,
    step:   usize,
    cursor: usize,
    random: Vec<u8>
}

impl<'a, T: 'a> RandomIter<'a, T> {
    pub fn new(iter: impl IntoIterator<Item=&'a T>) -> Self {
        let items = Vec::from_iter(iter);
        let cursor = items.len() - 1;

        // Determine how many bytes are needed to represent the largest index value. This will be how many random bytes
        // are requested per item.
        let mut step = 0;
        let mut max = cursor;

        while max != 0 {
            max >>= 8;
            step += 1;
        }

        let mut random = vec![0; items.len() * step];

        if getrandom::getrandom(&mut random).is_err() {
            panic!("getrandom() failed");
        }

        Self { items, cursor, step, random, done: false, }
    }

    #[inline]
    pub fn reset(&mut self) {
        if getrandom::getrandom(&mut self.random).is_err() {
            panic!("getrandom() failed");
        }

        self.cursor = self.items.len() - 1;
        self.done = false;
    }

    #[inline]
    fn next_index(&self) -> usize {
        let index = (self.items.len() - 1) - self.cursor;
        let value = &self.random[index .. index + self.step];

        // Could possibly be sped up with std::mem::transmute, but it would be at the cost of endianness/alignment
        // complexity.
        value.iter().fold(0, |acc, &b| acc << 8 | b as usize) % (self.cursor + 1)
    }
}

impl<'a, T:'a> FromIterator<&'a T> for RandomIter<'a, T> {
    fn from_iter<I: IntoIterator<Item=&'a T>>(iter: I) -> Self {
        Self::new(iter)
    }
}

impl<'a, T:'a> Iterator for RandomIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            self.reset();
            return None;
        }

        let next_index = self.next_index();
        self.items.swap(next_index, self.cursor);
        let random_item = self.items[self.cursor];

        if self.cursor > 0 {
            self.cursor -= 1;
        }
        else {
            self.done = true;
        }

        Some(random_item)
    }
}

impl<'a, T:'a, I> traits::Random<'a, T> for I where I: std::iter::Iterator<Item=&'a T> {
    fn random(self) -> RandomIter<'a, T> {
        RandomIter::new(self)
    }
}