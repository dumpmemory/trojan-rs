use pnet::packet::{
    ip::IpNextHeaderProtocol,
    ipv4::{Ipv4Packet, MutableIpv4Packet},
    ipv6::{Ipv6Packet, MutableIpv6Packet},
    Packet,
};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

pub enum IpPacket<'a> {
    V4(Ipv4Packet<'a>),
    V6(Ipv6Packet<'a>),
}

impl<'a> Packet for IpPacket<'a> {
    fn packet(&self) -> &[u8] {
        match self {
            IpPacket::V4(packet) => packet.packet(),
            IpPacket::V6(packet) => packet.packet(),
        }
    }

    fn payload(&self) -> &[u8] {
        match self {
            IpPacket::V4(packet) => packet.payload(),
            IpPacket::V6(packet) => packet.payload(),
        }
    }
}

impl<'a> IpPacket<'a> {
    pub fn new(data: &'a [u8]) -> Option<IpPacket<'a>> {
        if let Some(packet) = Ipv4Packet::new(data) {
            Some(IpPacket::V4(packet))
        } else if let Some(packet) = Ipv6Packet::new(data) {
            Some(IpPacket::V6(packet))
        } else {
            None
        }
    }

    pub fn get_source(&self) -> IpAddr {
        match self {
            IpPacket::V4(packet) => IpAddr::V4(packet.get_source()),
            IpPacket::V6(packet) => IpAddr::V6(packet.get_source()),
        }
    }

    pub fn get_destination(&self) -> IpAddr {
        match self {
            IpPacket::V4(packet) => IpAddr::V4(packet.get_destination()),
            IpPacket::V6(packet) => IpAddr::V6(packet.get_destination()),
        }
    }

    pub fn get_next_level_protocol(&self) -> IpNextHeaderProtocol {
        match self {
            IpPacket::V4(packet) => packet.get_next_level_protocol(),
            IpPacket::V6(packet) => packet.get_next_header(),
        }
    }
}

pub enum MutableIpPacket {
    V4(MutableIpv4Packet<'static>),
    V6(MutableIpv6Packet<'static>),
}

impl MutableIpPacket {
    pub fn new(data: &[u8], is_v6: bool) -> Option<MutableIpPacket> {
        let mut packet = if is_v6 {
            if let Some(packet) =
                MutableIpv6Packet::owned(vec![0u8; Ipv6Packet::minimum_packet_size() + data.len()])
            {
                Some(MutableIpPacket::V6(packet))
            } else {
                None
            }
        } else if let Some(packet) =
            MutableIpv4Packet::owned(vec![0u8; Ipv4Packet::minimum_packet_size() + data.len()])
        {
            Some(MutableIpPacket::V4(packet))
        } else {
            None
        };
        if let Some(packet) = &mut packet {
            packet.set_payload(data);
        }
        packet
    }

    pub fn set_source(&mut self, ip: IpAddr) {
        match self {
            MutableIpPacket::V4(packet) => packet.set_source(get_ipv4(ip)),
            MutableIpPacket::V6(packet) => packet.set_source(get_ipv6(ip)),
        }
    }

    pub fn set_destination(&mut self, ip: IpAddr) {
        match self {
            MutableIpPacket::V4(packet) => packet.set_destination(get_ipv4(ip)),
            MutableIpPacket::V6(packet) => packet.set_destination(get_ipv6(ip)),
        }
    }

    pub fn set_payload(&mut self, payload: &[u8]) {
        match self {
            MutableIpPacket::V4(packet) => packet.set_payload(payload),
            MutableIpPacket::V6(packet) => packet.set_payload(payload),
        }
    }

    pub fn into_immutable(self) -> IpPacket<'static> {
        match self {
            MutableIpPacket::V4(packet) => IpPacket::V4(packet.consume_to_immutable()),
            MutableIpPacket::V6(packet) => IpPacket::V6(packet.consume_to_immutable()),
        }
    }
}

fn get_ipv4(ip: IpAddr) -> Ipv4Addr {
    if let IpAddr::V4(ip) = ip {
        ip
    } else {
        panic!("invalid ip type, v4 required")
    }
}

fn get_ipv6(ip: IpAddr) -> Ipv6Addr {
    if let IpAddr::V6(ip) = ip {
        ip
    } else {
        panic!("invalid ip type, v6 required")
    }
}