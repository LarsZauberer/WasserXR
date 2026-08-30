//! Double-buffered storage for values that are produced and consumed in separate phases.
//!
//! [`RotationBuffer`] defines the behavior shared by all rotation-buffer implementations.
//! [`VecRotationBuffer`] is the built-in `Vec`-backed implementation and also exposes the
//! currently readable values as a slice and an iterator.

/// Separates pending values from values that are currently readable.
///
/// # Lifecycle
///
/// A newly created buffer is empty. Values passed to [`Self::push`] are pending and are not
/// readable until [`Self::rotate`] is called. Rotation replaces the readable values with the
/// values that were pushed since the previous rotation, preserving insertion order. Values that
/// were readable before the rotation are discarded.
///
/// [`Self::clear`] removes both pending and readable values. Calling [`Self::rotate`] when no
/// values are pending therefore leaves the buffer empty.
pub trait RotationBuffer<T> {
    /// Makes the pending values readable and discards the previously readable values.
    ///
    /// The lifecycle and ordering guarantees are defined by [`RotationBuffer`].
    fn rotate(&mut self);

    /// Appends `elem` to the pending values.
    ///
    /// The value becomes readable after the next [`Self::rotate`].
    fn push(&mut self, elem: T);

    /// Returns the readable value at `index`, or [`None`] when `index` is out of bounds.
    ///
    /// Values are indexed in insertion order within the most recent readable batch.
    fn get(&self, index: usize) -> Option<&T>;

    /// Returns the length of the readable elements
    fn len(&self) -> usize;

    /// Returns whether the buffer is empty or not
    fn is_empty(&self) -> bool;

    /// Removes all pending and readable values from the buffer.
    fn clear(&mut self);
}

#[derive(Clone, Debug)]
/// A `Vec`-backed implementation of [`RotationBuffer`].
///
/// The behavior of the buffer is defined by [`RotationBuffer`]. This implementation additionally
/// provides the readable values as a contiguous slice through [`Self::as_slice`] and as an
/// iterator through [`Self::iter`].
///
/// # Examples
///
/// ```
/// use wasserxr::utils::rotation_buffer::{RotationBuffer, VecRotationBuffer};
///
/// let mut buffer = VecRotationBuffer::new();
/// buffer.push(1);
/// assert!(buffer.as_slice().is_empty());
///
/// buffer.rotate();
/// assert_eq!(buffer.as_slice(), &[1]);
///
/// buffer.push(2);
/// assert_eq!(buffer.as_slice(), &[1]);
/// buffer.rotate();
/// assert_eq!(buffer.iter().copied().collect::<Vec<_>>(), vec![2]);
/// ```
#[derive(Default)]
pub struct VecRotationBuffer<T> {
    writing: Vec<T>,
    reading: Vec<T>,
}

impl<T> VecRotationBuffer<T> {
    /// Creates an empty rotation buffer.
    pub fn new() -> Self {
        Self {
            writing: Vec::new(),
            reading: Vec::new(),
        }
    }

    /// Returns the currently readable values as a contiguous slice.
    ///
    /// This contains the same values, in the same order, that are available through
    /// [`RotationBuffer::get`] and [`Self::iter`]. Pending values are not included.
    pub fn as_slice(&self) -> &[T] {
        self.reading.as_slice()
    }

    /// Returns an iterator over the currently readable values in insertion order.
    ///
    /// The iterator yields the same sequence as [`Self::as_slice`]. Pending values are not
    /// included.
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.reading.iter()
    }
}

impl<T> RotationBuffer<T> for VecRotationBuffer<T> {
    fn rotate(&mut self) {
        std::mem::swap(&mut self.writing, &mut self.reading);
        self.writing.clear();
    }

    fn push(&mut self, elem: T) {
        self.writing.push(elem);
    }

    fn get(&self, index: usize) -> Option<&T> {
        self.reading.get(index)
    }

    fn clear(&mut self) {
        self.writing.clear();
        self.reading.clear();
    }

    fn len(&self) -> usize {
        self.reading.len()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
