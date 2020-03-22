use std::path::PathBuf;
use std::str::FromStr;

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

use crate::errors::ConnectionToolsError;
use netstat2::{
    get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, SocketInfo,
    TcpSocketInfo, UdpSocketInfo,
};

pub struct SocketsContainer {
    pub tcp_sockets: Vec<(TcpSocketInfo, Vec<u32>)>,
    pub udp_sockets: Vec<(UdpSocketInfo, Vec<u32>)>,
}
impl SocketsContainer {
    pub fn new() -> Self {
        SocketsContainer {
            tcp_sockets: Vec::new(),
            udp_sockets: Vec::new(),
        }
    }
}

pub struct App<'a> {
    pub sockets_info_res: Result<SocketsContainer, ConnectionToolsError>,
    pub items: Vec<&'a str>,
    pub selected: Option<usize>,
    pub events: Vec<(&'a str, &'a str)>,
    pub info_style: Style,
    pub warning_style: Style,
    pub error_style: Style,
    pub critical_style: Style,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        App {
            sockets_info_res: Result::Ok(SocketsContainer::new()),
            items: vec![
                "Item1", "Item2", "Item3", "Item4", "Item5", "Item6", "Item7", "Item8", "Item9",
                "Item10", "Item11", "Item12", "Item13", "Item14", "Item15", "Item16", "Item17",
                "Item18", "Item19", "Item20", "Item21", "Item22", "Item23", "Item24",
            ],
            selected: None,
            events: vec![
                ("Event1", "INFO"),
                ("Event2", "INFO"),
                ("Event3", "CRITICAL"),
                ("Event4", "ERROR"),
                ("Event5", "INFO"),
                ("Event6", "INFO"),
                ("Event7", "WARNING"),
                ("Event8", "INFO"),
                ("Event9", "INFO"),
                ("Event10", "INFO"),
                ("Event11", "CRITICAL"),
                ("Event12", "INFO"),
                ("Event13", "INFO"),
                ("Event14", "INFO"),
                ("Event15", "INFO"),
                ("Event16", "INFO"),
                ("Event17", "ERROR"),
                ("Event18", "ERROR"),
                ("Event19", "INFO"),
                ("Event20", "INFO"),
                ("Event21", "WARNING"),
                ("Event22", "INFO"),
                ("Event23", "INFO"),
                ("Event24", "WARNING"),
                ("Event25", "INFO"),
                ("Event26", "INFO"),
            ],
            info_style: Style::default().fg(Color::White),
            warning_style: Style::default().fg(Color::Yellow),
            error_style: Style::default().fg(Color::Magenta),
            critical_style: Style::default().fg(Color::Red),
        }
    }

    pub fn advance(&mut self) {
        let event = self.events.pop().unwrap();
        self.events.insert(0, event);
    }

    pub fn update_sockets(&mut self) {
        let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
        let proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;
        let sockets_info = get_sockets_info(af_flags, proto_flags).map_err(|err| {
            ConnectionToolsError::FailToGetSocketsInfo {
                message: format!("{}", err),
            }
        });
        let tcp_and_upd_sockets = sockets_info.map(|sockets| split_sockets(sockets));
        self.sockets_info_res = tcp_and_upd_sockets;
    }
}

fn split_sockets(sockets_info: Vec<SocketInfo>) -> SocketsContainer {
    let sockets_len = sockets_info.len();
    let sockets_tuple = sockets_info.into_iter().fold(
        (
            Vec::with_capacity(sockets_len),
            Vec::with_capacity(sockets_len),
        ),
        |mut res_tuple, si| {
            match si {
                SocketInfo {
                    protocol_socket_info: ProtocolSocketInfo::Tcp(tcp_socket_info),
                    associated_pids,
                    ..
                } => res_tuple.0.push((tcp_socket_info, associated_pids)),

                SocketInfo {
                    protocol_socket_info: ProtocolSocketInfo::Udp(udp_socket_info),
                    associated_pids,
                    ..
                } => res_tuple.1.push((udp_socket_info, associated_pids)),
            }

            res_tuple
        },
    );
    //todo: should I shrink result vectors? Size they were created was too big
    SocketsContainer {
        tcp_sockets: sockets_tuple.0,
        udp_sockets: sockets_tuple.1,
    }
}
