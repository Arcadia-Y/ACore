// a simple range structure for type T
pub trait Step {
    fn step(&mut self);
}

#[derive(Clone, Copy)]
pub struct Range<T>
where
    T: Copy + Eq + Step,
{
    pub start: T,
    pub end: T,
}

pub struct RangeIterator<T> {
    curr: T,
    end: T,
}

impl<T> Range<T> 
where
    T: Copy + Eq + Step, 
{
    pub fn new(start: T, end: T) -> Self {
        Self { start, end }
    }

    pub fn iter(&self) -> RangeIterator<T> {
        RangeIterator {
            curr: self.start,
            end: self.end,
        }
    }
}

impl<T> Iterator for RangeIterator<T>
where
    T: Copy + Eq + Step,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr == self.end {
            None
        } else {
            let res = self.curr;
            self.curr.step();
            Some(res)
        }
    }
}
