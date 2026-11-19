#!/usr/bin/env python3
"""Writes clip.mkv, the test video of Q5.9a slice 4: Matroska with one
V_UNCOMPRESSED track (RGBA, 32 x 24, 5 frames in one second, frame k one
colour of COLOURS with a white square of 8 px that moves 4 px a frame) and
one A_PCM/INT/LIT track (16-bit mono, 8 kHz, a 440 Hz tone).

Run from anywhere: python3 -I browser/tests/media/make_clip.py
"""
import math
import os
import struct

COLOURS = [(255, 0, 0), (0, 255, 0), (0, 0, 255), (255, 255, 0), (0, 255, 255)]
W, H, RATE = 32, 24, 8000


def size(n):
    for width in range(1, 9):
        if n < (1 << (7 * width)) - 1:
            return ((1 << (7 * width)) | n).to_bytes(width, "big")
    raise ValueError(n)


def el(ident, body):
    return bytes.fromhex(ident) + size(len(body)) + body


def uint(ident, n):
    return el(ident, n.to_bytes(max(1, (n.bit_length() + 7) // 8), "big"))


def frame(k):
    r, g, b = COLOURS[k]
    out = bytearray()
    for y in range(H):
        for x in range(W):
            white = 4 * k <= x < 4 * k + 8 and y < 8
            out += bytes((255, 255, 255, 255) if white else (r, g, b, 255))
    return bytes(out)


def block(track, ms, data):
    return el("A3", size(track) + struct.pack(">hB", ms, 0x80) + data)


def main():
    head = el("1A45DFA3", uint("4286", 1) + uint("42F7", 1) + uint("42F2", 4) + uint("42F3", 8)
              + el("4282", b"matroska") + uint("4287", 4) + uint("4285", 2))
    info = el("1549A966", uint("2AD7B1", 1000000) + el("4489", struct.pack(">d", 1000.0)))
    video = el("AE", uint("D7", 1) + uint("73C5", 1) + uint("83", 1) + el("86", b"V_UNCOMPRESSED")
               + el("E0", uint("B0", W) + uint("BA", H) + el("2EB524", b"RGBA")))
    audio = el("AE", uint("D7", 2) + uint("73C5", 2) + uint("83", 2) + el("86", b"A_PCM/INT/LIT")
               + el("E1", el("B5", struct.pack(">d", float(RATE))) + uint("9F", 1) + uint("6264", 16)))
    blocks = b""
    for k in range(5):
        ms = 200 * k
        pcm = b"".join(struct.pack("<h", int(12000 * math.sin(2 * math.pi * 440 * (ms * RATE // 1000 + i) / RATE)))
                       for i in range(RATE // 5))
        blocks += block(1, ms, frame(k)) + block(2, ms, pcm)
    cluster = el("1F43B675", uint("E7", 0) + blocks)
    body = info + el("1654AE6B", video + audio) + cluster
    with open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "clip.mkv"), "wb") as f:
        f.write(head + el("18538067", body))


main()
