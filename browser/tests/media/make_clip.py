#!/usr/bin/env python3
"""Writes clip.mkv, the test video of Q5.9a slice 4: Matroska with one
V_UNCOMPRESSED track (RGBA, 32 x 24, 5 frames in one second, frame k one
colour of COLOURS with a white square of 8 px that moves 4 px a frame) and
one A_PCM/INT/LIT track (16-bit mono, 8 kHz, a 440 Hz tone).

Also stream.mkv and stream.json (see `stream`).

Run from anywhere: python3 -I browser/tests/media/make_clip.py
"""
import json
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


def tone(ms):
    return b"".join(struct.pack("<h", int(12000 * math.sin(2 * math.pi * 440 * (ms * RATE // 1000 + i) / RATE)))
                    for i in range(RATE // 5))


def stream(head, tracks, here):
    """stream.mkv: the same tracks in ten clusters of 200 ms (frame k has colour k % 5), and stream.json:
    where the initialization segment ends and where each cluster is, for the MediaSource tests."""
    info = el("1549A966", uint("2AD7B1", 1000000))
    out = head + bytes.fromhex("18538067") + b"\x01\xff\xff\xff\xff\xff\xff\xff" + info + tracks
    marks = {"init": len(out), "clusters": []}
    for k in range(10):
        cluster = el("1F43B675", uint("E7", 200 * k) + block(1, 0, frame(k % 5)) + block(2, 0, tone(200 * k)))
        marks["clusters"].append([len(out), len(out) + len(cluster)])
        out += cluster
    with open(os.path.join(here, "stream.mkv"), "wb") as f:
        f.write(out)
    with open(os.path.join(here, "stream.json"), "w") as f:
        json.dump(marks, f)
        f.write("\n")


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
        blocks += block(1, 200 * k, frame(k)) + block(2, 200 * k, tone(200 * k))
    cluster = el("1F43B675", uint("E7", 0) + blocks)
    here = os.path.dirname(os.path.abspath(__file__))
    body = info + el("1654AE6B", video + audio) + cluster
    with open(os.path.join(here, "clip.mkv"), "wb") as f:
        f.write(head + el("18538067", body))
    stream(head, el("1654AE6B", video + audio), here)


main()
