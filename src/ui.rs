use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, RwLock,
};
use std::time::Instant;

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, Gauge, GraphType, List, ListItem},
    Frame,
};

use crate::{IP_COUNT, SPEED_GRAPH_VALUES};

pub fn ui<B: Backend>(
    f: &mut Frame<B>,
    ui_events: Arc<RwLock<Vec<String>>>,
    ui_history: &[usize],
    ui_ip_count: Arc<AtomicUsize>,
    start: Instant,
) {
    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(93), Constraint::Percentage(7)])
        .split(f.size());
    let hchunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
        .split(vchunks[0]);
    let hchunks2 = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
        .split(hchunks[1]);

    let raw_data = ui_history.iter().rev().take(SPEED_GRAPH_VALUES).rev();
    let max_y = raw_data.clone().max().unwrap();
    let data = raw_data
        .enumerate()
        .map(|(i, x)| (i as f64, *x as f64))
        .collect::<Vec<_>>();
    let datasets = vec![Dataset::default()
        .marker(symbols::Marker::Dot)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Cyan))
        .data(&data)];

    let speed_graph_values_str = format!("-{}", SPEED_GRAPH_VALUES);
    let max_y_str = max_y.to_string();
    let speed = Chart::new(datasets)
        .block(Block::default().title("Speed").borders(Borders::ALL))
        .x_axis(
            Axis::default()
                .style(Style::default().fg(Color::White))
                .bounds([0.0, SPEED_GRAPH_VALUES as f64])
                .labels(
                    [&speed_graph_values_str, "0"]
                        .iter()
                        .cloned()
                        .map(Span::from)
                        .collect(),
                ),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::White))
                .bounds([0.0, *max_y as f64])
                .labels(["0", &max_y_str].iter().cloned().map(Span::from).collect()),
        );
    f.render_widget(speed, hchunks[0]);

    let log = List::new(
        ui_events
            .read()
            .unwrap()
            .iter()
            .rev()
            .map(|x| ListItem::new(x.to_owned()))
            .collect::<Vec<_>>(),
    )
    .block(Block::default().title("Log").borders(Borders::ALL));
    f.render_widget(log, hchunks2[0]);

    let ip_count = ui_ip_count.load(Ordering::Relaxed);
    let status = List::new(sys_status(ui_history, start, ip_count))
        .block(Block::default().title("Status").borders(Borders::ALL));
    f.render_widget(status, hchunks2[1]);

    let gauge = Gauge::default()
        .block(
            Block::default()
                .title("Percent Checked")
                .borders(Borders::ALL),
        )
        .gauge_style(
            Style::default()
                .fg(Color::Magenta)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .percent((ip_count / IP_COUNT) as u16);
    f.render_widget(gauge, vchunks[1]);
}

fn sys_status(ui_history: &[usize], start: Instant, ui_ip_count: usize) -> Vec<ListItem> {
    vec![
        format!("Ips Checked: {}", nice_num_str(ui_ip_count)),
        format!("Elapsed Time: {}", nice_time(start.elapsed().as_secs())),
        format!(
            "Current Speed: {}",
            nice_num_str(*ui_history.last().unwrap())
        ),
        format!(
            "Max Speed: {}",
            nice_num_str(*ui_history.iter().max().unwrap())
        ),
    ]
    .iter()
    .map(|x| ListItem::new(x.to_owned()))
    .collect::<Vec<_>>()
}

fn nice_time(time: u64) -> String {
    let hor = time / 3600;
    let min = (time - hor * 3600) / 60;
    let sec = time - min * 60 - hor * 3600;
    format!("{:0>2}:{:0>2}:{:0>2}", hor, min, sec)
}

fn nice_num_str(num: usize) -> String {
    let num = num.to_string().chars().rev().collect::<Vec<_>>();
    let mut out = String::new();
    for (i, e) in num.iter().enumerate() {
        if i % 3 == 0 && i != 0 {
            out.insert(0, ',');
        }
        out.insert(0, *e);
    }
    out
}
