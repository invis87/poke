use tui::backend::Backend;
use tui::layout::Corner;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, List, Paragraph, SelectableList, Text, Widget};
use tui::Frame;

use crate::app::App;

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    {
        let sockets_chunk = main_chunks[0];

        Block::default()
            .borders(Borders::ALL)
            .title("Open sockets")
            .render(f, sockets_chunk);

        let sockets_info_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(90), Constraint::Min(2)].as_ref())
            .split(sockets_chunk);

        let socket_connections_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(sockets_info_layout[0]);

        let tcp_sockets_layout = socket_connections_layout[0];
        let udp_sockets_layout = socket_connections_layout[1];
        let text_socket_info_layout = sockets_info_layout[1];

        SelectableList::default()
            .block(Block::default().title("TCP").borders(Borders::ALL))
            .items(&app.tcp_sockets)
            .select(app.selected)
            .highlight_style(
                Style::default()
                    .fg(Color::LightGreen)
                    .modifier(Modifier::BOLD),
            )
            .highlight_symbol(">")
            .render(f, tcp_sockets_layout);

        SelectableList::default()
            .block(Block::default().title("UDP").borders(Borders::ALL))
            .items(&app.udp_sockets)
            .select(app.selected)
            .highlight_style(
                Style::default()
                    .fg(Color::LightGreen)
                    .modifier(Modifier::BOLD),
            )
            .highlight_symbol(">")
            .render(f, udp_sockets_layout);

        let text = [Text::raw(format!(
            "TCP count: {}; UDP count: {}",
            app.tcp_sockets_count, app.udp_sockets_count
        ))];
        Paragraph::new(text.iter()).render(f, text_socket_info_layout);

        //todo: dead code, but I want to save it for later
        let is_error = false;
        if is_error {
            let error_message = "wow, error happens!";
            let text = [Text::styled(
                format!("{}", error_message),
                Style::default().fg(Color::Red),
            )];
            Paragraph::new(text.iter())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Error")
                        .title_style(Style::default().fg(Color::Magenta).modifier(Modifier::BOLD)),
                )
                .alignment(Alignment::Center)
                .wrap(true)
                .render(f, sockets_chunk);
        }
    }

    {
        let events = app.events.iter().map(|&(evt, level)| {
            Text::styled(
                format!("{}: {}", level, evt),
                match level {
                    "ERROR" => app.error_style,
                    "CRITICAL" => app.critical_style,
                    "WARNING" => app.warning_style,
                    _ => app.info_style,
                },
            )
        });

        List::new(events)
            .block(Block::default().borders(Borders::ALL).title("List"))
            .start_corner(Corner::BottomLeft)
            .render(f, main_chunks[1]);
    }
}
