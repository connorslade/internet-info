mod ip_iter;

fn main() {
    let ips = ip_iter::IpIter::new();
    for i in ips.into_iter() {}
}
