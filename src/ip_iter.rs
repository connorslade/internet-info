use std::fmt::{self, Display};
use std::io;
use std::iter::Iterator;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs};
use std::vec;

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

    pub fn to_ip_addr(&self) -> Ipv4Addr {
        Ipv4Addr::new(self.a, self.b, self.c, self.d)
    }
}

impl Iterator for IpIter {
    type Item = Self;

    fn next(&mut self) -> Option<Self> {
        if self.a == 255 && self.b == 255 && self.c == 255 && self.d == 255 {
            return None;
        }

        self.d += 1;
        max_reset(&mut self.c, &mut self.d);
        max_reset(&mut self.b, &mut self.c);
        max_reset(&mut self.a, &mut self.b);

        Some(self.clone())
    }
}

impl ToSocketAddrs for IpIter {
    type Iter = vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        Ok(vec![SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(self.a, self.b, self.c, self.d),
            80,
        ))]
        .into_iter())
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
