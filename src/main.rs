use std::io::{stdout, Write};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc,
};
use std::thread;

use crossbeam_channel;
use ureq;

mod ip_iter;

const THREAD_COUNT: usize = 4;
const DISPLAY_PER: usize = 100;

const IP_COUNT: usize = 4294967296;

enum Message {}

fn main() {
    let ips = ip_iter::IpIter::new().into_iter();
    let ip_count = AtomicUsize::new(0);
    let (tx, rx) = crossbeam_channel::unbounded();
    tx.send("HELLO").unwrap();

    thread::spawn(move || {
        for i in rx {
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

    for (i, e) in (0..THREAD_COUNT).enumerate() {
        let ip_iter = ips.skip(i * (IP_COUNT / THREAD_COUNT));
        let mut ip_stop_index = ((i + 1) * (IP_COUNT / THREAD_COUNT)) - 1;
        if e == THREAD_COUNT {
            ip_stop_index = IP_COUNT;
        }

        thread::spawn(move || {
            for (i, e) in ip_iter.enumerate() {
                if i >= ip_stop_index {
                    break;
                }

                let res = match ureq::get(&format!("http://{}/", e)).call() {
                    Ok(i) => i,
                    Err(_) => continue,
                };
            }
        });
    }
}
