use crate::base::{Length};
use std::iter::Sum;

#[test]
fn test_length_add() {
    let len1 = Length::from_mm(100);
    let len2 = Length::from_mm(200);
    let result = len1 + len2;
    assert_eq!(result, Length::from_mm(300));
}

#[test]
fn test_length_add_assign() {
    let mut len1 = Length::from_mm(100);
    let len2 = Length::from_mm(200);
    
    len1 += len2;
    assert_eq!(len1, Length::from_mm(300));
}

#[test]
fn test_length_sum() {
    let lengths = vec![Length::from_mm(100), Length::from_mm(200), Length::from_mm(300)];
    let total: Length = lengths.iter().copied().sum();
    assert_eq!(total, Length::from_mm(600));
}