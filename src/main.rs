use std::io::{stdout, Write};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc,
};
use std::thread;

use ureq;

mod ip_iter;

const IP_COUNT: usize = 4294967296;
const DISPLAY_PER: usize = 100;

fn main() {
    let ips = ip_iter::IpIter::new();
    let ip_count = AtomicUsize::new(0);
    let (rx, tx) = mpsc::channel();

    thread::spawn(move || {
        for i in tx {
            ip_count.fetch_add(1, Ordering::Relaxed);
            let current = ip_count.load(Ordering::Relaxed);
            let clean_current = current / IP_COUNT * DISPLAY_PER;

            print!(
                "\x1B[2K\r[{}{}] {}",
                "*".repeat(clean_current),
                " ".repeat(DISPLAY_PER - clean_current),
                i
            );
            stdout().flush().unwrap();
        }
    });

    for i in ips.into_iter() {
        rx.send(i).unwrap();
        let res = match ureq::get(&format!("http://{}/", i)).call() {
            Ok(i) => i,
            Err(_) => continue,
        };

        dbg!(res);
    }
}

// fn main() {
//     let timeout = Duration::from_secs(5);
//     let ips = ip_iter::IpIter::new();
//
//     let req = Request::builder()
//         .version(Version::HTTP_11)
//         .method("GET")
//         .header("User-Agent", "Internet-Info")
//         .body(())
//         .unwrap();
//
//     for i in ips.into_iter() {
//         let addr = i.to_socket_addrs().unwrap().next().unwrap();
//         let stream = TcpStream::connect_timeout(&addr, timeout).unwrap();
//         stream.write_all(req).unwrap();
//     }
// }
