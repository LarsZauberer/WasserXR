//! This module describes a simple implementation of a Ring Buffer which has a fixed capacity.

use std::collections::VecDeque;

use crate::utils::macros::invariant_msg;

#[derive(Debug, Clone)]
/// The `Ring` struct defines a simple ring buffer with a fixed size, where you can push stuff onto. Until the
/// capacity is reached, it acts similar to a normal `Vec`. After the capacity has been reached, new
/// elements pushed onto the ring, will remove the oldest elements.
///
/// It has a generic type which specifies what kind of type is stored inside of the ring. The
/// generic type doesn't require any trait implementations.
///
/// ## Invariants
///
/// - There are at most capacity many elements in the ring: [`Self::len()`] <= [`Self::cap()`]
///
/// ## Usage
///
/// ```rust
/// use wasserxr::utils::ring::Ring;
///
/// let mut ring: Ring<usize> = Ring::new(2); // provide a capacity
///
/// ring.push(1);
/// ring.push(2);
/// ring.push(3);
///
/// assert_eq!(*ring.get(0).unwrap(), 2);
/// assert_eq!(*ring.get(1).unwrap(), 3);
/// assert_eq!(ring.get(2), None);
/// ```
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

    /// Changes the ring capacity
    ///
    /// If the capacity is smaller than the old one, it will drop the n oldest elements till it is
    /// at the max capacity.
    /// If the capacity is larger than the old one, nothing special happens. These values will not
    /// be initialized. Furthermore, using [`Self::get`] or [`Self::get_mut`] will return `None` in case a value in
    /// the capacity but not yet pushed is being accessed.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use wasserxr::utils::ring::Ring;
    ///
    /// let mut ring: Ring<usize> = Ring::new(1);
    ///
    /// ring.push(1);
    /// ring.push(2);
    ///
    /// ring.set_capacity(2);
    ///
    /// assert_eq!(*ring.get(0).unwrap(), 2);
    /// assert_eq!(ring.get(1), None);
    /// ```
    pub fn set_capacity(&mut self, cap: usize) {
        self.check();
        if cap < self.cap {
            // New cap is smaller (need to throw away old log)
            for _ in 0..(self.cap - cap) {
                self.data.pop_front();
            }
        }

        self.cap = cap;
        self.check();
    }

    /// Appends a value to the ring. If the ring has reached it's capacity and the element would be
    /// larger than it's capacity, it drops the oldest value if the ring is full.
    pub fn push(&mut self, value: T) {
        self.check();
        if self.data.len() == self.cap {
            self.data.pop_front();
        }
        self.data.push_back(value);
        self.check();
    }

    /// Iterates over values from oldest to newest.
    pub fn iter(&self) -> std::collections::vec_deque::Iter<'_, T> {
        self.check();
        self.data.iter()
    }

    /// Same as [Self::iter] but with mutable references
    pub fn iter_mut(&mut self) -> std::collections::vec_deque::IterMut<'_, T> {
        self.check();
        self.data.iter_mut()
    }

    /// Returns the element at the index as a reference.
    /// In case that the index is out of bounds, it will return None
    ///
    /// Index 0 will point to the oldest element that is still stored with respect to the ring
    /// capacity.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.check();
        self.data.get(index)
    }

    /// Operates the same way as [`Self::get`] but instead returns a mutable reference.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.check();
        self.data.get_mut(index)
    }

    /// Drops all the elements in the ring. It keeps the same capacity.
    pub fn clear(&mut self) {
        self.check();
        self.data.clear();
    }

    // Return the capacity of the ring.
    pub fn cap(&self) -> usize {
        self.check();
        self.cap
    }

    // Return the length of the ring. It counts the amount of all the allocated objects.
    //
    // Invariant: [`Self::len()`] <= [`Self::cap()`]
    pub fn len(&self) -> usize {
        self.check();
        self.data.len()
    }

    /// Invariant checker to see if all the ring object is consistent with the invariants. The
    /// invariants are described in [`Self`]
    fn check(&self) {
        debug_assert!(
            self.data.len() <= self.cap,
            "{}",
            invariant_msg!("Ring data length is larger than the capacity")
        );
    }
}
