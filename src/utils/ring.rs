use std::collections::VecDeque;

#[derive(Clone)]
/// Fixed-capacity FIFO ring buffer.
pub struct Ring<T> {
    data: VecDeque<T>,
    cap: usize,
}

impl<T> Ring<T> {
    /// Creates an empty ring with capacity `cap`.
    pub fn new(cap: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(cap),
            cap,
        }
    }

    /// Changes the ring capacity and drops the oldest values if needed.
    pub fn set_capacity(&mut self, cap: usize) {
        if cap < self.cap {
            // New cap is smaller (need to throw away old log)
            for _ in 0..(self.cap - cap) {
                self.data.pop_front();
            }
        }

        self.cap = cap;
    }

    /// Appends a value, dropping the oldest value if the ring is full.
    pub fn push(&mut self, value: T) {
        if self.data.len() == self.cap {
            self.data.pop_front();
        }
        self.data.push_back(value);
    }

    /// Iterates over values from oldest to newest.
    pub fn iter(&self) -> std::collections::vec_deque::Iter<'_, T> {
        self.data.iter()
    }
}
