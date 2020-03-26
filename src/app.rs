use tui::style::{Color, Style};

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
    sockets_info_res: Result<SocketsContainer, ConnectionToolsError>,
    pub tcp_sockets: Vec<String>,
    pub udp_sockets: Vec<String>,
    pub tcp_sockets_count: usize,
    pub udp_sockets_count: usize,
    pub items: Vec<&'a str>,
    pub selected: Option<usize>,
    pub events: Vec<(&'a str, &'a str)>,
    pub info_style: Style,
    pub warning_style: Style,
    pub error_style: Style,
    pub critical_style: Style,
    pub should_quit: bool,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        App {
            sockets_info_res: Result::Ok(SocketsContainer::new()),
            tcp_sockets: Vec::new(),
            udp_sockets: Vec::new(),
            tcp_sockets_count: 0,
            udp_sockets_count: 0,
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
            should_quit: false,
        }
    }

    fn advance(&mut self) {
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
        let tcp_and_upd_sockets = sockets_info.map(split_sockets);
        self.sockets_info_res = tcp_and_upd_sockets;

        self.tcp_sockets_count = self
            .sockets_info_res
            .as_ref()
            .map(|sockets_container| sockets_container.tcp_sockets.len())
            .unwrap_or(0);
        self.udp_sockets_count = self
            .sockets_info_res
            .as_ref()
            .map(|sockets_container| sockets_container.udp_sockets.len())
            .unwrap_or(0);

        self.tcp_sockets = self
            .sockets_info_res
            .as_ref()
            .map(|sockets_container| {
                sockets_container
                    .tcp_sockets
                    .iter()
                    .map(|(tcp_si, pids)| tcp_socket_to_string(tcp_si, pids))
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();
        self.udp_sockets = self
            .sockets_info_res
            .as_ref()
            .map(|sockets_container| {
                sockets_container
                    .udp_sockets
                    .iter()
                    .map(|(udp_si, pids)| udp_socket_to_string(udp_si, pids))
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();
    }

    pub fn on_up(&mut self) {
        self.selected = if let Some(selected) = self.selected {
            if selected > 0 {
                Some(selected - 1)
            } else {
                Some(self.items.len() - 1)
            }
        } else {
            Some(0)
        }
    }

    pub fn on_down(&mut self) {
        self.selected = if let Some(selected) = self.selected {
            if selected >= self.items.len() - 1 {
                Some(0)
            } else {
                Some(selected + 1)
            }
        } else {
            Some(0)
        }
    }

    pub fn on_right(&mut self) {}

    pub fn on_left(&mut self) {
        self.selected = None;
    }

    pub fn on_key(&mut self, c: char) {
        match c {
            'q' => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    pub fn on_tick(&mut self) {
        //                self.advance();
        self.update_sockets();
    }
}

fn split_sockets(sockets_info: Vec<SocketInfo>) -> SocketsContainer {
    let sockets_len = sockets_info.len();
    let mut sockets_tuple = sockets_info.into_iter().fold(
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

    sockets_tuple.0.shrink_to_fit();
    sockets_tuple.1.shrink_to_fit();
    SocketsContainer {
        tcp_sockets: sockets_tuple.0,
        udp_sockets: sockets_tuple.1,
    }
}

fn tcp_socket_to_string(tcp_si: &TcpSocketInfo, associated_pids: &[u32]) -> String {
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

fn udp_socket_to_string(udp_si: &UdpSocketInfo, associated_pids: &[u32]) -> String {
    format!(
        "UDP local[{} : {}] -> *:* pids{:?}",
        udp_si.local_addr, udp_si.local_port, associated_pids
    )
}
