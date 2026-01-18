use std::ops::{Bound, Range, RangeBounds};

macro_rules! unwrap_or_return {
    ( $e:expr ) => {
        match $e {
            Some(x) => x,
            None => return,
        }
    };
}

macro_rules! unwrap_or_break {
    ( $e:expr ) => {
        match $e {
            Some(x) => x,
            None => break,
        }
    };
}

pub fn is_symbol(c: char) -> bool {
    let symbols = "$`\'\":~()\\+-=$#^[&]*<@%!{|}>/?.,";
    symbols.contains(c)
}

pub fn ranges_overlap<T: Ord>(a: &Range<T>, b: &Range<T>) -> bool {
    a.start <= b.end && b.start <= a.end
}
