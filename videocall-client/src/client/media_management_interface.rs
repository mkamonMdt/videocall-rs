use log::debug;
use protobuf::Message;
use types::protos::media_management_packet::{
    media_management_packet::EventType, MediaManagementPacket,
};
use types::protos::packet_wrapper::packet_wrapper::PacketType;
use types::protos::packet_wrapper::PacketWrapper;

use crate::client::send_packet::SendPacket;
use crate::crypto::aes::Aes128State;

#[derive(Debug)]
pub enum ManagementCommand {
    SubscribeTo(String),
    UnSubscribeFrom(String),
    NotifyOngoinStream,
}

pub fn send_command(
    packet_sender: &impl SendPacket,
    aes: &Aes128State,
    userid: String,
    cmd: ManagementCommand,
) {
    match cmd {
        ManagementCommand::SubscribeTo(peer) => {
            send_media_management_packet(packet_sender, aes, userid, peer, EventType::SUBSCRIBE);
        }
        ManagementCommand::UnSubscribeFrom(peer) => {
            send_media_management_packet(packet_sender, aes, userid, peer, EventType::UNSUBSCRIBE);
        }
        ManagementCommand::NotifyOngoinStream => {
            send_media_management_packet(
                packet_sender,
                aes,
                userid.clone(),
                userid,
                EventType::ONGOING_STREAM,
            );
        }
    }
}

fn send_media_management_packet(
    packet_sender: &impl SendPacket,
    aes: &Aes128State,
    userid: String,
    peer: String,
    event: EventType,
) {
    debug!(
        "Sending media management event={} for peer={}",
        event.to_string(),
        peer
    );
    let packet = prepare_media_management_packet(aes, userid, peer, event);
    packet_sender.send_packet(packet);
}

fn prepare_media_management_packet(
    aes: &Aes128State,
    userid: String,
    email: String,
    event: EventType,
) -> PacketWrapper {
    let packet = MediaManagementPacket {
        event_type: event.into(),
        email,
        ..Default::default()
    };
    let data = aes.encrypt(&packet.write_to_bytes().unwrap()).unwrap();
    PacketWrapper {
        data,
        email: userid,
        packet_type: PacketType::MEDIA_MANAGEMENT.into(),
        ..Default::default()
    }
}
