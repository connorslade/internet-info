use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, RwLock,
};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, List, ListItem},
    Frame,
};

use crate::{IP_COUNT};

pub fn ui<B: Backend>(
    f: &mut Frame<B>,
    ui_events: Arc<RwLock<Vec<String>>>,
    ui_ip_count: Arc<AtomicUsize>,
) {
    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(93), Constraint::Percentage(7)])
        .split(f.size());
    let hchunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
        .split(vchunks[0]);

    // let datasets = vec![Dataset::default()
    //     .marker(symbols::Marker::Dot)
    //     .graph_type(GraphType::Line)
    //     .style(Style::default().fg(Color::Cyan))
    //     .data(&[(0.0, 5.0), (1.0, 6.0), (1.5, 6.434)])];
    //
    // let speed = Chart::new(datasets)
    //     .block(Block::default().title("Speed").borders(Borders::ALL))
    //     .x_axis(
    //         Axis::default()
    //             .style(Style::default().fg(Color::White))
    //             .bounds([0.0, 10.0])
    //             .labels(
    //                 ["0.0", "5.0", "10.0"]
    //                     .iter()
    //                     .cloned()
    //                     .map(Span::from)
    //                     .collect(),
    //             ),
    //     )
    //     .y_axis(
    //         Axis::default()
    //             .style(Style::default().fg(Color::White))
    //             .bounds([0.0, 10.0])
    //             .labels(
    //                 ["0.0", "5.0", "10.0"]
    //                     .iter()
    //                     .cloned()
    //                     .map(Span::from)
    //                     .collect(),
    //             ),
    //     );
    // f.render_widget(speed, hchunks[0]);

    let log = List::new(
        ui_events
            .read()
            .unwrap()
            .iter()
            .map(|x| ListItem::new(x.to_owned()))
            .collect::<Vec<_>>(),
    )
    .block(Block::default().title("Log").borders(Borders::ALL));
    f.render_widget(log, hchunks[1]);

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL))
        .gauge_style(
            Style::default()
                .fg(Color::Magenta)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .percent((ui_ip_count.load(Ordering::Relaxed) / IP_COUNT) as u16);
    f.render_widget(gauge, vchunks[1]);
}
