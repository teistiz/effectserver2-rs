# Just a test script.

import socket


class LightControl(object):
    """Example client for the light server."""

    def __init__(self, nick, host='valot.party', port=9909):
        self.host = host
        self.port = port
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.nick = nick.encode('utf-8')

    def set(self, lights):
        """Lights should be a list of tuples like (id, r, g, b) with the
        RGB values in a [0..1] range."""

        packet = bytearray([
            1,  # API version
            0,  # Nick tag
        ])
        packet.extend(bytearray([ord(char) for char in self.nick]))
        packet.append(0)  # end nick tag

        # [0..1] -> [0..255]
        to_ubyte = lambda val: int(max(min(val, 1.0), 0.0) * 255)

        for light in lights:
            packet.extend([
                1,
                light[0],
                0,                      # Effect type
                to_ubyte(light[1]),     # r
                to_ubyte(light[2]),     # g
                to_ubyte(light[3]),     # b
            ])

        self.sock.sendto(packet, (self.host, self.port))


control = LightControl('airzero', 'localhost')

import math
import time

hue = 0
t = 0

def red(n, t):
    return 0.5 + math.sin(n + t) * 0.5

def green(n, t):
    return 0.5 + math.sin(n + 1.0 + t) * 0.5

def blue(n, t):
    return 0.5 + math.sin(n + 2.0 + t) * 0.5

while True:
    control.set([
        (n, red(n, t), green(n, t), blue(n, t)) for n in range(0, 28)])
    t += 0.1
    time.sleep(0.05)
