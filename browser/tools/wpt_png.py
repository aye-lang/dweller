"""PNG decoding and pixel comparison for the reftest runner."""
import struct
import zlib

CHANNELS = {0: 1, 2: 3, 4: 2, 6: 4}


def chunks(data):
    at = 8
    while at < len(data):
        size, kind = struct.unpack(">I4s", data[at:at + 8])
        yield kind, data[at + 8:at + 8 + size]
        at += 12 + size


def paeth(a, b, c):
    p = a + b - c
    pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
    if pa <= pb and pa <= pc:
        return a
    return b if pb <= pc else c


def unfilter_row(kind, row, prev, bpp):
    out = bytearray(row)
    for i in range(len(out)):
        left = out[i - bpp] if i >= bpp else 0
        up = prev[i]
        upleft = prev[i - bpp] if i >= bpp else 0
        add = (0, left, up, (left + up) // 2, paeth(left, up, upleft))[kind]
        out[i] = (out[i] + add) & 255
    return out


def decode(data):
    """(width, height, channels, rows) of a non-interlaced 8-bit PNG."""
    width = height = depth = color = 0
    idat = b""
    for kind, body in chunks(data):
        if kind == b"IHDR":
            width, height, depth, color = struct.unpack(">IIBB", body[:10])
        elif kind == b"IDAT":
            idat += body
    if depth != 8 or color not in CHANNELS:
        raise ValueError(f"unsupported PNG (depth {depth}, color {color})")
    bpp = CHANNELS[color]
    raw = zlib.decompress(idat)
    stride = width * bpp
    rows, prev = [], bytearray(stride)
    for y in range(height):
        at = y * (stride + 1)
        prev = unfilter_row(raw[at], raw[at + 1:at + 1 + stride], prev, bpp) if raw[at] else raw[at + 1:at + 1 + stride]
        rows.append(bytes(prev))
    return width, height, bpp, rows


def row_difference(a, b, bpp):
    """(differing pixels, largest channel difference) of two rows."""
    count = peak = 0
    for x in range(0, len(a), bpp):
        if a[x:x + bpp] != b[x:x + bpp]:
            count += 1
            peak = max(peak, max(abs(p - q) for p, q in zip(a[x:x + bpp], b[x:x + bpp])))
    return count, peak


def difference(png_a, png_b):
    """(differing pixels, largest channel difference), or None when the
    images differ in size."""
    wa, ha, bpp, rows_a = decode(png_a)
    wb, hb, bppb, rows_b = decode(png_b)
    if (wa, ha, bpp) != (wb, hb, bppb):
        return None
    count = peak = 0
    for a, b in zip(rows_a, rows_b):
        if a != b:
            c, p = row_difference(a, b, bpp)
            count += c
            peak = max(peak, p)
    return count, peak
