//! A queue of fixed size.

pub struct Ring<T, const N: usize> {
    slots: [Option<T>; N],
    head: usize,
    len: usize,
}

impl<T, const N: usize> Ring<T, N> {
    pub fn new() -> Self {
        Self { slots: [const { None }; N], head: 0, len: 0 }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.len == N {
            return Err(value);
        }
        self.slots[(self.head + self.len) % N] = Some(value);
        self.len += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        let value = self.slots[self.head].take()?;
        self.head = (self.head + 1) % N;
        self.len -= 1;
        Some(value)
    }
}
