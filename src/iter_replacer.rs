use std::collections::VecDeque;
use std::borrow::Borrow;

/// An iterator that consumes another iterator and replaces every matching
/// sequence with a different sequence.
///
/// Replacer will stop reading from the source iterator once it has received
/// the first None, otherwise it would not be able to output tails that
/// are less than the size of the replace pattern.
///
/// For example imagine you're replacing [1, 2, 3], and the input is
/// [1, 2, 3, 1, 2]. Since Replacer would get a None from the source stream
/// after the 2, it wouldn't know if it should output [1, 2] or wait until
/// it gets the next value from the source. By closing the input source
/// once it has received a None, Replacer will know that the [1, 2] are
/// NOT an incomplete [1, 2, 3] pattern and it will be able to return
/// the [1, 2] immediately (maybe this behavior should be made configurable?)
pub struct Replacer<'a, I, A, B>
where I: Iterator,
{
    iter: Option<I>,
    q: VecDeque<B>,
    a: &'a [A],
    b: &'a [B],
    replacing: Option<usize>,
}
impl<'a, I, A, B> Replacer<'a, I, A, B>
where I: Iterator,
      I::Item: Borrow<B>,
      B: Clone + PartialEq<A>,
{
    /// Creates a new replacer
    ///
    /// # Panics
    ///
    /// Panics if the 'a' slice is empty
    pub fn new(iter: I, a: &'a [A], b: &'a [B]) -> Self {
        assert!(a.len() > 0, "the 'a' slice cannot be empty");
        Replacer {
            iter: Some(iter),
            q: VecDeque::with_capacity(a.len()),
            a,
            b,
            replacing: None,
        }
    }

    fn fill_q(&mut self) {
        let iter = match &mut self.iter {
            Some(x) => x,
            None => return,
        };
        while self.q.len() < self.a.len() {
            if let Some(x) = iter.next() {
                self.q.push_back(x.borrow().to_owned());
            } else {
                self.iter = None;
                return;
            }
        }
    }
    fn q_starts_with(&self) -> bool {
        for (q, a) in self.q.iter().zip(self.a.iter()) {
            if q != a {
                return false;
            }
        }

        self.q.len() == self.a.len()
    }
}
impl<'a, I, A, B> Iterator for Replacer<'a, I, A, B>
where I: Iterator,
      I::Item: Borrow<B>,
      B: Clone + PartialEq<A>,
{

    type Item = B;

    fn next(&mut self) -> Option<B> {
        if let Some(ref mut i) = self.replacing {
            if let Some(x) = self.b.get(*i) {
                *i += 1;
                return Some(x.to_owned());
            } else {
                self.replacing = None;
            }
        }

        self.fill_q();

        if self.q_starts_with() {
            self.q.clear();
            self.replacing = Some(0);
            return self.next();
        }

        self.q.pop_front()
    }
}

pub trait ReplacerExt<'a, I, A, B>
where I: Iterator,
      I::Item: Borrow<B>,
      B: Clone + PartialEq<A>,
{
    fn replacer(self, a: &'a [A], b: &'a [B]) -> Replacer<'a, I, A, B>;
}
impl<'a, I, A, B> ReplacerExt<'a, I, A, B> for I
where I: Iterator,
      I::Item: Borrow<B>,
      B: Clone + PartialEq<A>,
{
    fn replacer(self, a: &'a [A], b: &'a [B]) -> Replacer<'a, I, A, B> {
        Replacer::new(self, a, b)
    }
}

