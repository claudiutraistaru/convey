extern crate pnet;

use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::tcp::MutableTcpPacket;
use pnet::packet::{tcp, Packet};
use pnet::packet::ipv4::{MutableIpv4Packet};
use std::net::{IpAddr, Ipv4Addr};
use pnet::datalink::{NetworkInterface};
use pnet::packet::ethernet::{EtherTypes, MutableEthernetPacket, EthernetPacket};
use pnet::datalink::MacAddr;

const ETHERNET_HEADER_LEN: usize = 14;
const IPV4_HEADER_LEN: usize = 20;
const TCP_HEADER_LEN: usize = 32;

fn find_local_addr() -> Option<IpAddr> {
    for iface in pnet::datalink::interfaces() {
        if !iface.is_loopback() {
            for ipnet in iface.ips {
                if ipnet.is_ipv4() {
                    return Some(ipnet.ip());
                }
            }
        }
    }
    return None;
}

// only use in tests!
pub fn build_dummy_ip(src_ip: Ipv4Addr, dst_ip: Ipv4Addr, src_port: u16, dst_port: u16) -> MutableIpv4Packet<'static> {
    // Setup TCP header
    let mut vec: Vec<u8> = vec![0; TCP_HEADER_LEN];
    let mut tcp_header = MutableTcpPacket::new(&mut vec[..]).unwrap();

    tcp_header.set_source(src_port);
    tcp_header.set_destination(dst_port);

    tcp_header.set_flags(tcp::TcpFlags::SYN);
    tcp_header.set_window(64240);
    tcp_header.set_data_offset(8);
    tcp_header.set_urgent_ptr(0);
    tcp_header.set_sequence(rand::random::<u32>());

    let checksum = pnet::packet::tcp::ipv4_checksum(&tcp_header.to_immutable(), &src_ip, &dst_ip);
    tcp_header.set_checksum(checksum);

    // Setup IP header
    let ipbuf: Vec<u8> = vec!(0; TCP_HEADER_LEN + IPV4_HEADER_LEN);
    let mut ip_header = MutableIpv4Packet::owned(ipbuf).unwrap();
    ip_header.set_header_length(69);
    ip_header.set_total_length(52);
    ip_header.set_fragment_offset(16384);
    ip_header.set_next_level_protocol(IpNextHeaderProtocols::Tcp);
    ip_header.set_source(src_ip);
    ip_header.set_destination(dst_ip);
    ip_header.set_identification(rand::random::<u16>());
    ip_header.set_ttl(128);
    ip_header.set_version(4);
    ip_header.set_payload(&tcp_header.packet());

    let checksum = pnet::packet::ipv4::checksum(&ip_header.to_immutable());
    ip_header.set_checksum(checksum);

    return ip_header;
}

// only use in tests!
pub fn build_dummy_eth(src_ip: Ipv4Addr, dst_ip: Ipv4Addr, src_port: u16, dst_port: u16) -> MutableEthernetPacket<'static> {
    let ip_header = build_dummy_ip(src_ip, dst_ip, src_port, dst_port);

    // Setup Ethernet header
    let ethbuf: Vec<u8> = vec!(0; TCP_HEADER_LEN + IPV4_HEADER_LEN + ETHERNET_HEADER_LEN);
    let mut eth_header = MutableEthernetPacket::owned(ethbuf).unwrap();

    eth_header.set_destination(MacAddr::new(255, 255, 255, 255, 255, 255));
    eth_header.set_source(MacAddr::new(255, 255, 255, 255, 255, 255));
    eth_header.set_ethertype(EtherTypes::Ipv4);
    eth_header.set_payload(&ip_header.packet());

    return eth_header;
}
