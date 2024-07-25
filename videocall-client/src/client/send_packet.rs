use types::protos::packet_wrapper::PacketWrapper;

pub trait SendPacket {
    fn send_packet(&self, packet: PacketWrapper);
}
