///
/// Connection struct wraps the lower-level "Task" (task.rs), providing a heartbeat and keeping
/// track of connection status.
///
use super::task::Task;
use super::ConnectOptions;
use crate::crypto::aes::Aes128State;
use gloo::timers::callback::Interval;
use log::{error, info};
use protobuf::Message;
use std::rc::Rc;
use types::protos::media_packet::media_packet::MediaType;
use types::protos::media_packet::MediaPacket;
use types::protos::packet_wrapper::packet_wrapper::PacketType;
use types::protos::packet_wrapper::PacketWrapper;
use yew::prelude::Callback;

#[derive(Debug)]
pub struct Connecting {
    userid: String,
    task: Rc<Task>,
    aes: Rc<Aes128State>,
    peer_monitor: Callback<()>,
}

#[derive(Debug)]
pub struct Connected {
    task: Rc<Task>,
    _heartbeat: Interval,
    _heartbeat_monitor: Interval,
}

#[derive(Debug)]
pub struct Closed {}

#[derive(Debug)]
pub enum Connection {
    Closed(Closed),
    Connecting(Connecting),
    Connected(Connected),
}

impl Connection {
    pub fn new() -> Self {
        Connection::Closed(Closed {})
    }

    pub fn connect(
        &self,
        webtransport: bool,
        options: ConnectOptions,
        aes: Rc<Aes128State>,
    ) -> Self {
        match self {
            Connection::Closed(_) => {
                let userid = options.userid.clone();
                let peer_monitor = options.peer_monitor.clone();
                match Task::connect(webtransport, options) {
                    Ok(task) => {
                        let task = Rc::new(task);
                        Connection::Connecting(Connecting {
                            userid,
                            task,
                            aes,
                            peer_monitor,
                        })
                    }
                    Err(_) => Connection::Closed(Closed {}),
                }
            }
            _ => {
                error!("Unable to connect - terminating connection");
                self.disconnect()
            }
        }
    }

    pub fn complate_connection(&self) -> Self {
        match self {
            Connection::Connecting(state) => Connection::Connected(Connected::from(state)),
            _ => {
                error!("Unable to complate connection - not in Connecting state");
                Connection::Closed(Closed {})
            }
        }
    }

    pub fn disconnect(&self) -> Self {
        info!("Connection terminated");
        Connection::new()
    }

    pub fn is_connected(&self) -> bool {
        matches!(self, Connection::Connected(_))
    }

    pub fn send_packet(&self, packet: PacketWrapper) {
        if let Connection::Connected(state) = self {
            state.task.send_packet(packet);
        }
    }
}

impl Connected {
    fn start_heartbeat(task: Rc<Task>, aes: Rc<Aes128State>, userid: String) -> Interval {
        Interval::new(1000, move || {
            let packet = MediaPacket {
                media_type: MediaType::HEARTBEAT.into(),
                email: userid.clone(),
                timestamp: js_sys::Date::now(),
                ..Default::default()
            };
            let data = aes.encrypt(&packet.write_to_bytes().unwrap()).unwrap();
            let packet = PacketWrapper {
                data,
                email: userid.clone(),
                packet_type: PacketType::MEDIA_MANDATORY.into(),
                ..Default::default()
            };

            task.send_packet(packet);
        })
    }

    fn from(val: &Connecting) -> Connected {
        let task = Rc::clone(&val.task);
        let aes = Rc::clone(&val.aes);
        let peer_monitor = val.peer_monitor.clone();
        Connected {
            task: Rc::clone(&val.task),
            _heartbeat: Self::start_heartbeat(task, aes, val.userid.clone()),
            _heartbeat_monitor: Interval::new(5000, move || {
                peer_monitor.emit(());
            }),
        }
    }
}
