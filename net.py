"""
Host in the middle Mininet setup
┌───────────────┐     ┌───────────────┐      ┌───────────────┐
│               │     │               │      │               │
│               │     │               │      │               │
│   Left Host  ◄├─────┤► Middle Host ◄├──────┤►  Right Host  │
│               │     │               │      │               │
│               │     │               │      │               │
└───────────────┘     └───────────────┘      └───────────────┘
"""

import subprocess

from mininet.cli import CLI
from mininet.link import TCLink
from mininet.net import Mininet
from mininet.node import OVSController
from mininet.topo import Topo

LEFT_HOST_NAME: str = 'left_h'
MIDDLE_HOST_NAME: str = 'middle_h'
RIGHT_HOST_NAME: str = 'right_h'


class HostInTheMiddleTopo(Topo):

    def build(self):

        left_host = self.addHost(LEFT_HOST_NAME)
        #middle_host = self.addSwitch("s1")
        right_host = self.addHost(RIGHT_HOST_NAME)

        #self.addLink(left_host, middle_host, bw=1000, delay="100ms")
        #self.addLink(middle_host, right_host, bw=100, delay="100ms")
        self.addLink(left_host, right_host, bw=100, delay="100ms")


def get_interface(host: str, index: int):
    return f"{host}-eth{index}"


def disable_offload(host, offload: str, index: int):
    host.cmd(f"ethtool -K {get_interface(host.name, index)} {offload} off")


def drop_ip(host, index):
    host.cmd(f"ip addr flush dev {get_interface(host.name, index)}")
    host.cmd(f"ip -6 addr flush dev {get_interface(host.name, index)}")


def run():
    net = Mininet(topo=HostInTheMiddleTopo(),
                  controller=OVSController, link=TCLink)

    left_host = net[LEFT_HOST_NAME]
    right_host = net[RIGHT_HOST_NAME]

    for host in [left_host, right_host]:
        for name in ["tx", "rx"]:
            disable_offload(host, name, 0)

    left_host.cmd("tcpdump -w left.pcap &")
    right_host.cmd("tcpdump -w right.pcap &")

    left_host.cmd(f"yes | nc -l 0.0.0.0 8000 &")
    left_host.cmd(f"sudo ./target/release/smol-speed > out.txt 2> err.txt &")

    net.start()   
    CLI(net)
    net.stop()


if __name__ == "__main__":
    run()

