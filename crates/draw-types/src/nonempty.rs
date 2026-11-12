//! [`NonEmpty`]: a vector that always holds at least one element, so `first`
//! and `last` cannot fail.

use std::ops::Deref;

/// A `Vec<T>` with at least one element.
#[derive(Debug, Clone, PartialEq)]
pub struct NonEmpty<T> {
    items: Vec<T>,
}

impl<T> NonEmpty<T> {
    /// A one-element vector.
    #[must_use]
    pub fn new(first: T) -> Self {
        Self { items: vec![first] }
    }

    /// Wraps `items`, or `None` if it is empty.
    #[must_use]
    pub fn from_vec(items: Vec<T>) -> Option<Self> {
        (!items.is_empty()).then_some(Self { items })
    }

    /// The first element.
    #[must_use]
    pub fn first(&self) -> &T {
        &self.items[0]
    }

    /// The last element.
    #[must_use]
    pub fn last(&self) -> &T {
        &self.items[self.items.len() - 1]
    }

    /// Appends an element.
    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    /// Removes the last element unless it is the only one.
    pub fn pop(&mut self) -> Option<T> {
        if self.items.len() > 1 {
            self.items.pop()
        } else {
            None
        }
    }

    /// Number of elements, at least one.
    #[must_use]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// The elements as a slice.
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        &self.items
    }

    /// Gives the elements back as a plain vector.
    #[must_use]
    pub fn into_vec(self) -> Vec<T> {
        self.items
    }
}

impl<T> Deref for NonEmpty<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonempty_from_vec_rejects_empty() {
        assert!(NonEmpty::<i32>::from_vec(Vec::new()).is_none());
        assert!(NonEmpty::from_vec(vec![1]).is_some());
    }

    #[test]
    fn nonempty_first_last_are_total() {
        let v = NonEmpty::from_vec(vec![1, 2, 3]).unwrap_or_else(|| NonEmpty::new(0));

        assert_eq!((*v.first(), *v.last()), (1, 3));
        assert_eq!(NonEmpty::new(7).first(), NonEmpty::new(7).last());
    }

    #[test]
    fn nonempty_pop_keeps_one_element() {
        let mut v = NonEmpty::new(1);
        v.push(2);

        assert_eq!(v.pop(), Some(2));
        assert_eq!(v.pop(), None);
        assert_eq!(v.len(), 1);
        assert_eq!(v.as_slice(), &[1]);
    }

    #[test]
    fn nonempty_into_vec_returns_elements_in_order() {
        let v = NonEmpty::from_vec(vec![4, 5]).unwrap_or_else(|| NonEmpty::new(0));

        assert_eq!(v.into_vec(), vec![4, 5]);
    }
}
