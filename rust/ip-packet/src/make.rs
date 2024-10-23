//! Factory module for making all kinds of packets.

use crate::{IpPacket, IpPacketBuf};
use anyhow::{bail, Context, Result};
use etherparse::PacketBuilder;
use std::net::IpAddr;

/// Helper macro to turn a [`PacketBuilder`] into an [`IpPacket`].
#[macro_export]
macro_rules! build {
    ($packet:expr, $payload:ident) => {{
        let size = $packet.size($payload.len());
        let mut ip = $crate::IpPacketBuf::new();

        $packet
            .write(&mut std::io::Cursor::new(ip.buf()), &$payload)
            .expect("Buffer should be big enough");

        IpPacket::new(ip, size).expect("Should be a valid IP packet")
    }};
}

pub fn fz_p2p_control(header: [u8; 8], control_payload: &[u8]) -> Result<IpPacket> {
    let ip_payload_size = header.len() + control_payload.len();

    anyhow::ensure!(ip_payload_size <= crate::PACKET_SIZE);

    let builder = etherparse::PacketBuilder::ipv6(
        crate::fz_p2p_control::ADDR.octets(),
        crate::fz_p2p_control::ADDR.octets(),
        0,
    );
    let packet_size = builder.size(ip_payload_size);

    let mut packet_buf = IpPacketBuf::new();

    let mut payload_buf = vec![0u8; 8 + control_payload.len()];
    payload_buf[..8].copy_from_slice(&header);
    payload_buf[8..].copy_from_slice(control_payload);

    builder
        .write(
            &mut std::io::Cursor::new(packet_buf.buf()),
            crate::fz_p2p_control::IP_NUMBER,
            &payload_buf,
        )
        .with_context(|| {
            format!("Buffer should be big enough; ip_payload_size={ip_payload_size}")
        })?;
    let ip_packet = IpPacket::new(packet_buf, packet_size).context("Unable to create IP packet")?;

    Ok(ip_packet)
}

pub fn icmp_request_packet(
    src: IpAddr,
    dst: impl Into<IpAddr>,
    seq: u16,
    identifier: u16,
    payload: &[u8],
) -> Result<IpPacket> {
    match (src, dst.into()) {
        (IpAddr::V4(src), IpAddr::V4(dst)) => {
            let packet = PacketBuilder::ipv4(src.octets(), dst.octets(), 64)
                .icmpv4_echo_request(identifier, seq);

            Ok(build!(packet, payload))
        }
        (IpAddr::V6(src), IpAddr::V6(dst)) => {
            let packet = PacketBuilder::ipv6(src.octets(), dst.octets(), 64)
                .icmpv6_echo_request(identifier, seq);

            Ok(build!(packet, payload))
        }
        _ => bail!(IpVersionMismatch),
    }
}

pub fn icmp_reply_packet(
    src: IpAddr,
    dst: impl Into<IpAddr>,
    seq: u16,
    identifier: u16,
    payload: &[u8],
) -> Result<IpPacket> {
    match (src, dst.into()) {
        (IpAddr::V4(src), IpAddr::V4(dst)) => {
            let packet = PacketBuilder::ipv4(src.octets(), dst.octets(), 64)
                .icmpv4_echo_reply(identifier, seq);

            Ok(build!(packet, payload))
        }
        (IpAddr::V6(src), IpAddr::V6(dst)) => {
            let packet = PacketBuilder::ipv6(src.octets(), dst.octets(), 64)
                .icmpv6_echo_reply(identifier, seq);

            Ok(build!(packet, payload))
        }
        _ => bail!(IpVersionMismatch),
    }
}

pub fn echo_reply(mut req: IpPacket) -> Option<IpPacket> {
    if !req.is_udp() && !req.is_tcp() {
        return None;
    }

    if let Some(mut packet) = req.as_tcp_mut() {
        let original_src = packet.get_source_port();
        let original_dst = packet.get_destination_port();

        packet.set_source_port(original_dst);
        packet.set_destination_port(original_src);
    }

    if let Some(mut packet) = req.as_udp_mut() {
        let original_src = packet.get_source_port();
        let original_dst = packet.get_destination_port();

        packet.set_source_port(original_dst);
        packet.set_destination_port(original_src);
    }

    let original_src = req.source();
    let original_dst = req.destination();

    req.set_dst(original_src);
    req.set_src(original_dst);

    Some(req)
}

pub fn tcp_packet<IP>(
    saddr: IP,
    daddr: IP,
    sport: u16,
    dport: u16,
    payload: Vec<u8>,
) -> Result<IpPacket>
where
    IP: Into<IpAddr>,
{
    match (saddr.into(), daddr.into()) {
        (IpAddr::V4(src), IpAddr::V4(dst)) => {
            let packet =
                PacketBuilder::ipv4(src.octets(), dst.octets(), 64).tcp(sport, dport, 0, 128);

            Ok(build!(packet, payload))
        }
        (IpAddr::V6(src), IpAddr::V6(dst)) => {
            let packet =
                PacketBuilder::ipv6(src.octets(), dst.octets(), 64).tcp(sport, dport, 0, 128);

            Ok(build!(packet, payload))
        }
        _ => bail!(IpVersionMismatch),
    }
}

pub fn udp_packet<IP>(
    saddr: IP,
    daddr: IP,
    sport: u16,
    dport: u16,
    payload: Vec<u8>,
) -> Result<IpPacket>
where
    IP: Into<IpAddr>,
{
    match (saddr.into(), daddr.into()) {
        (IpAddr::V4(src), IpAddr::V4(dst)) => {
            let packet = PacketBuilder::ipv4(src.octets(), dst.octets(), 64).udp(sport, dport);

            Ok(build!(packet, payload))
        }
        (IpAddr::V6(src), IpAddr::V6(dst)) => {
            let packet = PacketBuilder::ipv6(src.octets(), dst.octets(), 64).udp(sport, dport);

            Ok(build!(packet, payload))
        }
        _ => bail!(IpVersionMismatch),
    }
}

#[derive(thiserror::Error, Debug)]
#[error("IPs must be of the same version")]
pub struct IpVersionMismatch;
