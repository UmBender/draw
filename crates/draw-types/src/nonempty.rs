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
