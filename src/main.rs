#![allow(clippy::collapsible_if)]

use std::cmp;
use std::os::fd::AsRawFd;

use smoltcp::iface::{Config, Interface, SocketSet};
use smoltcp::phy::{wait as phy_wait, Device, Medium};
use smoltcp::socket::tcp::{self, CongestionControl};
use smoltcp::time::{Duration, Instant};
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr};

const AMOUNT: usize = 1_000_000_000;

fn main() {
    let mut device = smoltcp::phy::RawSocket::new("left_h-eth0", Medium::Ethernet).unwrap();
    let fd = device.as_raw_fd();

    const BUF_SIZE: usize = 65535000;

    let tcp1_rx_buffer = tcp::SocketBuffer::new(vec![b'A'; BUF_SIZE]);
    let tcp1_tx_buffer = tcp::SocketBuffer::new(vec![b'A'; BUF_SIZE]);
    let mut tcp1_socket = tcp::Socket::new(tcp1_rx_buffer, tcp1_tx_buffer);

    tcp1_socket.set_congestion_control(CongestionControl::Cubic);

    let mut config = match device.capabilities().medium {
        Medium::Ethernet => {
            Config::new(EthernetAddress([0xd0, 0x57, 0x7b, 0x11, 0xc9, 0x05]).into())
        }
        Medium::Ip => Config::new(smoltcp::wire::HardwareAddress::Ip),
        Medium::Ieee802154 => todo!(),
    };
    config.random_seed = rand::random();

    let mut iface = Interface::new(config, &mut device, Instant::now());
    iface.update_ip_addrs(|ip_addrs| {
        ip_addrs
            .push(IpCidr::new(IpAddress::v4(10, 0, 0, 5), 24))
            .unwrap();
    });

    let mut sockets = SocketSet::new(vec![]);
    let tcp1_handle = sockets.add(tcp1_socket);
    let default_timeout = Some(Duration::from_millis(1000));

    let mut processed = 0;
    while processed < AMOUNT {
        let timestamp = Instant::now();
        iface.poll(timestamp, &mut device, &mut sockets);

        // tcp:1234: emit data
        let socket = sockets.get_mut::<tcp::Socket>(tcp1_handle);
        if !socket.is_open() {
            socket.listen(8000).unwrap();
        }

        if socket.can_send() {
            if processed < AMOUNT {
                let length = socket
                    .send(|buffer| {
                        let length = cmp::min(buffer.len(), AMOUNT - processed);
                        (length, length)
                    })
                    .unwrap();
                processed += length;
            }
        }

        match iface.poll_at(timestamp, &sockets) {
            Some(poll_at) if timestamp < poll_at => {
                phy_wait(fd, Some(poll_at - timestamp)).expect("wait error");
            }
            Some(_) => (),
            None => {
                phy_wait(fd, default_timeout).expect("wait error");
            }
        }
    }
}
