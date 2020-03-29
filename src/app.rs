use tui::style::{Color, Style};

use crate::errors::ConnectionToolsError;
use netstat2::{
    get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, SocketInfo,
    TcpSocketInfo, UdpSocketInfo,
};
use sysinfo::{ProcessExt, SystemExt};

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

pub struct App {
    sockets_info_res: Result<SocketsContainer, ConnectionToolsError>,
    pub tcp_sockets: Vec<String>,
    pub udp_sockets: Vec<String>,
    pub tcp_sockets_count: usize,
    pub udp_sockets_count: usize,
    pub selected_type: SelectedType,
    tcp_selection: Option<usize>,
    udp_selection: Option<usize>,
    pub info_style: Style,
    pub warning_style: Style,
    pub error_style: Style,
    pub critical_style: Style,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> App {
        App {
            sockets_info_res: Result::Ok(SocketsContainer::new()),
            tcp_sockets: Vec::new(),
            udp_sockets: Vec::new(),
            tcp_sockets_count: 0,
            udp_sockets_count: 0,
            selected_type: SelectedType::Nothing,
            tcp_selection: None,
            udp_selection: None,
            info_style: Style::default().fg(Color::White),
            warning_style: Style::default().fg(Color::Yellow),
            error_style: Style::default().fg(Color::Magenta),
            critical_style: Style::default().fg(Color::Red),
            should_quit: false,
        }
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
                self.tcp_selection = up_select_counter(&self.tcp_selection, &self.tcp_sockets_count)
            }
            SelectedType::Udp => {
                self.udp_selection = up_select_counter(&self.udp_selection, &self.udp_sockets_count)
            }
        }
    }

    pub fn on_down(&mut self) {
        match self.selected_type {
            SelectedType::Nothing => (),
            SelectedType::Tcp => {
                self.tcp_selection =
                    down_select_counter(&self.tcp_selection, &self.tcp_sockets_count)
            }
            SelectedType::Udp => {
                self.udp_selection =
                    down_select_counter(&self.udp_selection, &self.udp_sockets_count)
            }
        }
    }

    pub fn selected_tcp(&self) -> Option<usize> {
        match self.selected_type {
            SelectedType::Udp => None,
            SelectedType::Nothing => None,
            SelectedType::Tcp => self.tcp_selection,
        }
    }

    pub fn selected_udp(&self) -> Option<usize> {
        match self.selected_type {
            SelectedType::Nothing => None,
            SelectedType::Tcp => None,
            SelectedType::Udp => self.udp_selection,
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
        self.update_sockets();
    }

    pub fn selected_socket_info(&self) -> String {
        match self.selected_type {
            SelectedType::Nothing => "choose socket with arrow keys".to_owned(),
            SelectedType::Tcp => match &self.sockets_info_res {
                Err(_) => "fail to get sockets info".to_owned(),
                Ok(sockets_info) => {
                    let selected_socket =
                        &sockets_info.tcp_sockets[self.tcp_selection.unwrap_or(0)];
                    let pids = &selected_socket.1;

                    //todo: move systemInfo outside
                    let mut system = sysinfo::System::new_all();

                    // First we update all information of our system struct.
                    system.refresh_all();

                    // Now let's print every process' id and name:
                    let pids_info = pids
                        .iter()
                        .map(|&pid| {
                            system
                                .get_process(pid as i32)
                                .map(|proc_| {
                                    format!(
                                        "pid {}::\nname {}\nstatus: {:?}\ncmd: {:?}\nexe: {:?}\nenviron: {:?}\nmemory: {}\nvirtual memory: {}\nstart time: {}\ncpu usage: {}",
                                        pid,
                                        proc_.name(),
                                        proc_.status(),
                                        proc_.cmd(),
                                        proc_.exe(),
                                        proc_.environ(),
                                        proc_.memory(),
                                        proc_.virtual_memory(),
                                        proc_.start_time(),
                                        proc_.cpu_usage(),
                                    )
                                })
                                .unwrap_or("todo: fix me".to_owned())
                        })
                        .collect();
                    pids_info
                }
            },
            SelectedType::Udp => "todo: implement in the same way as for TCP".to_owned(),
        }
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
        "local[{} : {}] -> remote [{} : {}]; pids{:?}; state: {}",
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
        "local[{} : {}] -> *:* pids{:?}",
        udp_si.local_addr, udp_si.local_port, associated_pids
    )
}

fn up_select_counter(current: &Option<usize>, base_collection_len: &usize) -> Option<usize> {
    if let Some(current) = current.as_ref() {
        if *current > 0 {
            Some(*current - 1)
        } else {
            Some(*base_collection_len - 1)
        }
    } else {
        Some(0)
    }
}

fn down_select_counter(current: &Option<usize>, base_collection_len: &usize) -> Option<usize> {
    if let Some(current) = current.as_ref() {
        if *current >= *base_collection_len - 1 {
            Some(0)
        } else {
            Some(*current + 1)
        }
    } else {
        Some(0)
    }
}
