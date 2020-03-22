#[macro_use]
extern crate failure;

mod app;
mod errors;
mod util;

use app::App;

use std::io;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::Corner;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, List, Paragraph, SelectableList, Text, Widget};
use tui::Terminal;

use crate::util::event::{Event, Events};
use netstat2::{
    get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, SocketInfo,
    TcpSocketInfo, UdpSocketInfo,
};

fn main() -> Result<(), failure::Error> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = Events::new();

    // App
    let mut app = App::new();
    terminal.clear();

    loop {
        terminal.draw(|mut f| {
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(f.size());

            {
                let sockets_chunk = main_chunks[0];

                Block::default()
                    .borders(Borders::ALL)
                    .title("Open sockets")
                    .render(&mut f, sockets_chunk);

                match app.sockets_info_res.as_ref() {
                    Ok(sockets_container) => {
                        let socket_connections_layout = Layout::default()
                            .direction(Direction::Horizontal)
                            .margin(1)
                            .constraints(
                                [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
                            )
                            .split(sockets_chunk);

                        let tcp_sockets_layout = socket_connections_layout[0];
                        let udp_sockets_layout = socket_connections_layout[1];

                        let tcp_sockets_str = sockets_container
                            .tcp_sockets
                            .iter()
                            .map(|(tcp_si, pids)| tcp_socket_to_string(tcp_si, pids))
                            .collect::<Vec<String>>();

                        let udp_sockets_str = sockets_container
                            .udp_sockets
                            .iter()
                            .map(|(udp_si, pids)| udp_socket_to_string(udp_si, pids))
                            .collect::<Vec<String>>();

                        SelectableList::default()
                            .block(Block::default().title("TCP").borders(Borders::ALL))
                            .items(&tcp_sockets_str)
                            .select(app.selected)
                            .highlight_style(
                                Style::default()
                                    .fg(Color::LightGreen)
                                    .modifier(Modifier::BOLD),
                            )
                            .highlight_symbol(">")
                            .render(&mut f, tcp_sockets_layout);

                        SelectableList::default()
                            .block(Block::default().title("UDP").borders(Borders::ALL))
                            .items(&udp_sockets_str)
                            .select(app.selected)
                            .highlight_style(
                                Style::default()
                                    .fg(Color::LightGreen)
                                    .modifier(Modifier::BOLD),
                            )
                            .highlight_symbol(">")
                            .render(&mut f, udp_sockets_layout);
                    }

                    Err(error) => {
                        let text = [Text::styled(
                            format!("{}", error),
                            Style::default().fg(Color::Red),
                        )];
                        Paragraph::new(text.iter())
                            .block(
                                Block::default()
                                    .borders(Borders::ALL)
                                    .title("Error")
                                    .title_style(
                                        Style::default()
                                            .fg(Color::Magenta)
                                            .modifier(Modifier::BOLD),
                                    ),
                            )
                            .alignment(Alignment::Center)
                            .wrap(true)
                            .render(&mut f, sockets_chunk);
                    }
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
                    .render(&mut f, main_chunks[1]);
            }
        })?;

        match events.next()? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }
                Key::Left => {
                    app.selected = None;
                }
                Key::Down => {
                    app.selected = if let Some(selected) = app.selected {
                        if selected >= app.items.len() - 1 {
                            Some(0)
                        } else {
                            Some(selected + 1)
                        }
                    } else {
                        Some(0)
                    }
                }
                Key::Up => {
                    app.selected = if let Some(selected) = app.selected {
                        if selected > 0 {
                            Some(selected - 1)
                        } else {
                            Some(app.items.len() - 1)
                        }
                    } else {
                        Some(0)
                    }
                }
                _ => {}
            },
            Event::Tick => {
                //                app.advance();
                app.update_sockets();
            }
        }
    }
    Ok(())
}

fn tcp_socket_to_string(tcp_si: &TcpSocketInfo, associated_pids: &Vec<u32>) -> String {
    format!(
        "TCP local[{} : {}] -> remote [{} : {}]; pids{:?}; state: {}",
        tcp_si.local_addr,
        tcp_si.local_port,
        tcp_si.remote_addr,
        tcp_si.remote_port,
        associated_pids,
        tcp_si.state
    )
}

fn udp_socket_to_string(udp_si: &UdpSocketInfo, associated_pids: &Vec<u32>) -> String {
    format!(
        "UDP local[{} : {}] -> *:* pids{:?}",
        udp_si.local_addr, udp_si.local_port, associated_pids
    )
}
