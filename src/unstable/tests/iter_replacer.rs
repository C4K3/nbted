use crate::iter_replacer::ReplacerExt;

use std::sync::mpsc::sync_channel;

#[test]
fn noop() {
    let a: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];

    let b: Vec<u8> = a.iter().replacer(&[9], &[]).collect();
    assert_eq!(a, b);
}

#[test]
fn length_one_match() {
    let a: Vec<&str> = vec!["a", "b", "c", "d", "e", "f", "c"];
    let b: Vec<&str> = a.iter().replacer(&["c"], &["abc", "xyz"]).collect();
    assert_eq!(b, &["a", "b", "abc", "xyz", "d", "e", "f", "abc", "xyz"]);
}

#[test]
fn empty_replacement() {
    let a: Vec<Vec<&str>> = vec![vec![], vec!["a", "b"], vec![], vec![], vec!["c"], vec![]];
    let empty: Vec<&str> = vec![];
    let b: Vec<Vec<&str>> = a.iter().replacer(&[empty], &[]).collect();
    assert_eq!(b, vec![vec!["a", "b"], vec!["c"]]);
}

#[test]
fn long_replacement() {
    let a: Vec<i32> = vec![0, -1, -2, -3, 4, 5, 6, 7, 0, -1];
    let b: Vec<i32> = a
        .iter()
        .replacer(&[0], &[10, 11, 12, 13, 14, 15, 16])
        .collect();
    assert_eq!(
        &b,
        &[10, 11, 12, 13, 14, 15, 16, -1, -2, -3, 4, 5, 6, 7, 10, 11, 12, 13, 14, 15, 16, -1]
    );
}

#[test]
fn overlapping_match() {
    let a: Vec<i32> = vec![0, 1, 0, 1, 0, 1, 0, 1, 0, 1];
    let b: Vec<i32> = a.iter().replacer(&[0, 1, 0], &[0]).collect();
    assert_eq!(&b, &[0, 1, 0, 1, 0, 1]);
}

#[test]
fn incomplete_match() {
    let a: Vec<i32> = vec![0, 1, 2, 3, 4, 5, 1, 2];
    let b: Vec<i32> = a.iter().replacer(&[1, 2, 3], &[6]).collect();
    assert_eq!(&b, &[0, 6, 4, 5, 1, 2]);
}

#[test]
fn empty() {
    let a: Vec<i32> = vec![];
    let b: Vec<i32> = a.iter().replacer(&[1], &[2]).collect();
    assert_eq!(&a, &b);
}

#[test]
fn no_complete() {
    let a: Vec<i32> = vec![1, 2];
    let b: Vec<i32> = a.iter().replacer(&[1, 2, 3], &[0]).collect();
    assert_eq!(&a, &b);
}

#[test]
fn fuse() {
    let (tx, rx) = sync_channel(10);
    let mut iter = rx.try_iter().replacer(&[1, 2, 3], &[6, 7]);
    for x in &[0u8, 1, 2, 3, 4, 5, 1, 2] {
        tx.send(x).unwrap();
    }

    let b: Vec<u8> = iter.by_ref().collect();
    assert_eq!(&b, &[0, 6, 7, 4, 5, 1, 2]);

    tx.send(&3).unwrap();
    assert_eq!(iter.next(), None);
}

#[test]
#[should_panic]
fn empty_replace_string() {
    let a: Vec<u8> = vec![0, 1];
    let _ = a.iter().replacer(&[], &[1]);
}
