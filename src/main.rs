use std::io::{self, Write};
use std::net::Ipv4Addr;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, RwLock,
};
use std::thread;

use crossbeam_channel;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};
use ureq;

mod ip_iter;
mod ui;

pub const THREAD_COUNT: usize = 4;
pub const IP_COUNT: usize = 4_294_967_296; // 256^4

enum Message {
    IpCheck(Ipv4Addr),
    ThreadExit(usize),
}

fn main() {
    let ips = ip_iter::IpIter::new().into_iter();
    let ip_count = Arc::new(AtomicUsize::new(0));
    let threads_count = Arc::new(AtomicUsize::new(THREAD_COUNT));
    let events = Arc::new(RwLock::new(vec!["Starting".to_owned()]));
    let (tx, rx) = crossbeam_channel::unbounded();

    let ui_events = events.clone();
    let ui_ip_count = ip_count.clone();
    thread::spawn(move || {
        enable_raw_mode().unwrap();
        let mut stdout = io::stdout();
        stdout
            .write(&Clear(ClearType::All).to_string().as_bytes())
            .unwrap();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| ui::ui(f, ui_events, ui_ip_count))
            .unwrap();

        disable_raw_mode().unwrap();
        execute!(terminal.backend_mut(), LeaveAlternateScreen,).unwrap();
        terminal.show_cursor().unwrap();
    });

    thread::spawn(move || {
        for i in rx {
            match i {
                Message::IpCheck(_) => {
                    ip_count.fetch_add(1, Ordering::Relaxed);
                }
                Message::ThreadExit(i) => {
                    threads_count.fetch_sub(1, Ordering::Relaxed);
                    events.write().unwrap().push(format!("Thread Exit [{}]", i));
                }
            };
        }
    });

    for (i, e) in (0..THREAD_COUNT).enumerate() {
        let tx = tx.clone();
        let ip_iter = ips.skip(i * (IP_COUNT / THREAD_COUNT));
        let mut ip_stop_index = ((i + 1) * (IP_COUNT / THREAD_COUNT)) - 1;
        if e == THREAD_COUNT {
            ip_stop_index = IP_COUNT;
        }

        thread::spawn(move || {
            for (i, e) in ip_iter.enumerate() {
                if i >= ip_stop_index {
                    tx.send(Message::ThreadExit(i)).unwrap();
                    break;
                }

                let res = match ureq::get(&format!("http://{}/", e)).call() {
                    Ok(i) => i,
                    Err(_) => continue,
                };
            }
        });
    }

    thread::park();
}
