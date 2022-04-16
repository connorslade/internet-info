use std::fmt::{self, Display};
use std::iter::Iterator;

// a.b.c.d
#[derive(Debug, Clone, Copy)]
pub struct IpIter {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
}

impl IpIter {
    pub fn new() -> Self {
        IpIter {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
        }
    }
}

impl Iterator for IpIter {
    type Item = Self;

    fn next(&mut self) -> Option<Self> {
        self.d += 1;
        max_reset(&mut self.c, &mut self.d);
        max_reset(&mut self.b, &mut self.c);
        max_reset(&mut self.a, &mut self.b);

        Some(self.clone())
    }
}

impl Display for IpIter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.a, self.b, self.c, self.d,)
    }
}

fn max_reset(lower: &mut u8, this: &mut u8) {
    if *this == 255 {
        *this = 0;
        *lower += 1;
    }
}
