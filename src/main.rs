use std::fs;
use std::io::{self, Write};
use std::net::Ipv4Addr;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, RwLock,
};
use std::time::{Duration, Instant};
use std::{process, thread};

use crossbeam_channel;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, LeaveAlternateScreen},
};
use isahc::{config::Configurable, RequestExt};
use tui::{backend::CrosstermBackend, Terminal};

mod ip_iter;
mod ui;

pub const OUT_PATH: &str = "out.dat";
pub const UI_FPS: usize = 10;
pub const SPEED_GRAPH_VALUES: usize = 30;
pub const THREAD_COUNT: usize = 25;

pub const IP_COUNT: usize = 4_294_967_296; // 256^4

enum Message {
    IpCheck(Ipv4Addr, bool),
    ThreadExit(usize),
}

fn main() {
    let ips = ip_iter::IpIter::new().into_iter();
    let mspf = ((UI_FPS as f32).recip() * 1000.0) as u64;

    let ip_count_og = Arc::new(AtomicUsize::new(0));
    let threads_count = Arc::new(AtomicUsize::new(THREAD_COUNT));
    let events_og = Arc::new(RwLock::new(vec![format!("Starting [{}]", THREAD_COUNT)]));
    let real_ips = Arc::new(RwLock::new(Vec::new()));
    let (tx, rx) = crossbeam_channel::unbounded();
    println!("Loading...");

    fs::write("out.dat", "LOADING").unwrap();
    let events = events_og.clone();
    let ip_count = ip_count_og.clone();
    thread::spawn(move || {
        for i in rx {
            match i {
                Message::IpCheck(i, x) => {
                    if x {
                        real_ips.write().unwrap().push(i);
                    }

                    ip_count.fetch_add(1, Ordering::Relaxed);
                }
                Message::ThreadExit(i) => {
                    threads_count.fetch_sub(1, Ordering::Relaxed);
                    events.write().unwrap().push(format!("Thread Exit [{}]", i));
                    if threads_count.load(Ordering::Relaxed) == 0 {
                        let bin = bincode::serialize(&*real_ips.read().unwrap()).unwrap();
                        fs::write(OUT_PATH, bin).unwrap();

                        process::exit(0);
                    }
                }
            };
        }
    });

    for (ti, e) in (0..THREAD_COUNT).enumerate() {
        let tx = tx.clone();
        let ip_iter = ips.skip(ti * (IP_COUNT / THREAD_COUNT));
        let mut ip_stop_index = ((ti + 1) * (IP_COUNT / THREAD_COUNT)) - 1;
        if e == THREAD_COUNT {
            ip_stop_index = IP_COUNT;
        }

        thread::spawn(move || {
            for (i, e) in ip_iter.enumerate() {
                if i >= ip_stop_index {
                    tx.send(Message::ThreadExit(ti)).unwrap();
                    break;
                }

                let res = isahc::Request::get(&format!("http://{}/", e))
                    .timeout(Duration::from_millis(100))
                    .body(())
                    .unwrap()
                    .send();
                tx.send(Message::IpCheck(e.to_ip_addr(), res.is_ok()))
                    .unwrap();
            }
        });
    }

    enable_raw_mode().unwrap();
    let events = events_og.clone();
    let mut stdout = io::stdout();
    let mut ui_history = vec![0; SPEED_GRAPH_VALUES];
    let mut frame = 0;
    stdout
        .write(&Clear(ClearType::All).to_string().as_bytes())
        .unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    loop {
        let start = Instant::now();

        if frame % UI_FPS == 0 {
            let last: usize = ui_history.iter().sum();
            let new = ip_count_og.load(Ordering::Relaxed) - last;
            ui_history.push(new);
        }

        terminal
            .draw(|f| ui::ui(f, events_og.clone(), &ui_history, ip_count_og.clone()))
            .unwrap();
        frame += 1;

        let frame_time = start.elapsed().as_millis() as u64;
        if frame_time > mspf {
            events
                .write()
                .unwrap()
                .push(format!("Frametime too long [{}]", frame_time));
        }

        if crossterm::event::poll(Duration::from_millis(mspf.saturating_sub(frame_time))).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                match key.code {
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode().unwrap();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
    terminal.show_cursor().unwrap();
}
