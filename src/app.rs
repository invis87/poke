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

#[derive(Debug)]
pub enum SelectedType {
    Nothing,
    Tcp,
    Udp,
}
impl SelectedType {
    fn left(&self) -> Self {
        match &self {
            SelectedType::Nothing => SelectedType::Nothing,
            SelectedType::Tcp => SelectedType::Nothing,
            SelectedType::Udp => SelectedType::Tcp,
        }
    }

    fn right(&self) -> Self {
        match &self {
            SelectedType::Nothing => SelectedType::Tcp,
            SelectedType::Tcp => SelectedType::Udp,
            SelectedType::Udp => SelectedType::Udp,
        }
    }
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
    pub selected_type: SelectedType,
    pub selected_tcp: usize,
    pub selected_udp: usize,
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
            selected_type: SelectedType::Nothing,
            selected_tcp: 0,
            selected_udp: 0,
            events: vec![("Event1", "INFO"), ("Event2", "INFO"), ("Event26", "INFO")],
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
        match self.selected_type {
            SelectedType::Nothing => (),
            SelectedType::Tcp => {
                self.selected_tcp = up_select_counter(&self.selected_tcp, &self.tcp_sockets_count)
            }
            SelectedType::Udp => {
                self.selected_udp = up_select_counter(&self.selected_udp, &self.udp_sockets_count)
            }
        }
    }

    pub fn on_down(&mut self) {
        match self.selected_type {
            SelectedType::Nothing => (),
            SelectedType::Tcp => {
                self.selected_tcp = down_select_counter(&self.selected_tcp, &self.tcp_sockets_count)
            }
            SelectedType::Udp => {
                self.selected_udp = down_select_counter(&self.selected_udp, &self.udp_sockets_count)
            }
        }
    }

    pub fn on_right(&mut self) {
        self.selected_type = self.selected_type.right();
    }

    pub fn on_left(&mut self) {
        self.selected_type = self.selected_type.left();
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

fn up_select_counter(current: &usize, base_collection_len: &usize) -> usize {
    if *current > 0 {
        current - 1
    } else {
        base_collection_len - 1
    }
}

fn down_select_counter(current: &usize, base_collection_len: &usize) -> usize {
    if *current >= base_collection_len - 1 {
        0
    } else {
        current + 1
    }
}
