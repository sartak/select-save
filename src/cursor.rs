use std::cmp::{max, min};

pub struct Cursor {
    len: usize,
    page_size: usize,
    scroll_off: usize,
    index: usize,
    first: usize,
    last: usize,
}

impl Cursor {
    pub fn new(len: usize, page_size: usize) -> Self {
        let last = min(page_size, len) - 1;
        let scroll_off = page_size / 4;
        Self {
            len,
            page_size,
            scroll_off,
            index: 0,
            first: 0,
            last,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn visible_items(&self) -> usize {
        self.last - self.first + 1
    }

    fn go(&mut self, amount: isize) {
        let idx = self.index;

        if amount.abs() == 1 {
            self.index = (self.index as isize + self.len as isize + amount) as usize % self.len;
        } else if amount < 0 {
            let amount = (-amount) as usize;
            if self.index > amount {
                self.index -= amount;
            } else if self.index > 0 {
                self.index = 0;
            } else {
                self.index = self.len - 1;
            }
        } else if amount > 0 {
            let amount = amount as usize;
            if self.index + amount < self.len {
                self.index += amount;
            } else if self.index < self.len - 1 {
                self.index = self.len - 1;
            } else {
                self.index = 0;
            }
        }

        let first = if self.index < idx {
            if self.index as isize - self.scroll_off as isize >= self.first as isize {
                return;
            }
            self.index as isize - self.scroll_off as isize
        } else {
            if self.index + self.scroll_off <= self.last {
                return;
            }
            self.index as isize + self.scroll_off as isize - self.page_size as isize + 1
        };

        self.first = min(max(first, 0) as usize, self.len - self.page_size);
        self.last = min(self.first + self.page_size, self.len) - 1;
    }

    pub fn down(&mut self) {
        self.go(1);
    }

    pub fn up(&mut self) {
        self.go(-1);
    }

    pub fn page_down(&mut self) {
        self.go(self.page_size as isize);
    }

    pub fn page_up(&mut self) {
        self.go(-(self.page_size as isize));
    }

    pub fn iter<I>(&self, iter: impl Iterator<Item = I>) -> impl Iterator<Item = (bool, I)> {
        let min = self.first;
        let current = self.index - min;
        iter.skip(min)
            .take(self.last - min + 1)
            .enumerate()
            .map(move |(i, x)| (i == current, x))
    }
}
