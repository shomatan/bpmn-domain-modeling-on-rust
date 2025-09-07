use serde::{Deserialize, Serialize};

/// A vector that guarantees at least one element exists
/// Implements "Make Invalid States Unrepresentable" principle
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonEmptyVec<T> {
    head: T,
    tail: Vec<T>,
}

impl<T> NonEmptyVec<T> {
    /// Create a new NonEmptyVec with a single element
    pub fn new(head: T) -> Self {
        Self {
            head,
            tail: Vec::new(),
        }
    }

    /// Get the first element (guaranteed to exist)
    pub fn first(&self) -> &T {
        &self.head
    }

    /// Add an element to the end
    pub fn push(&mut self, value: T) {
        self.tail.push(value);
    }

    /// Get the length (always >= 1)
    pub fn len(&self) -> usize {
        1 + self.tail.len()
    }

    /// Iterator over all elements
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        std::iter::once(&self.head).chain(self.tail.iter())
    }
}

impl<T> From<T> for NonEmptyVec<T> {
    fn from(head: T) -> Self {
        Self::new(head)
    }
}