"""Builds core/data/ephem.bin and core/src/deltat_table.rs from public astronomical data.

Usage: python tools/build_ephem.py <source folder>

The folder has to hold:
  de440.bsp          https://ssd.jpl.nasa.gov/ftp/eph/planets/bsp/de440.bsp
  chiron.txt         JPL Horizons, target 2060, barycentric ICRF vectors in AU, 1799-06-01 to 2400-06-01, step 4 d
  dt_horizons.txt    JPL Horizons, target 10, quantity 30 (TDB-UT), CSV, step 12 mo, 1800-01-01 to 2400-01-01

ephem.bin keeps JPL's Chebyshev series, cut to the years the program calculates. Coefficients are
rounded to a fixed step and bit-packed at a fixed width per coefficient, so the program can read any
segment straight out of the file without unpacking anything.

File layout, little endian:
  u32 body count
  per body: u32 id, u32 segments, u32 coefficients, u32 data offset, f64 start JD, f64 segment days,
            f64 km per step, then 3 * coefficients u8 bit widths
  packed data
"""
import re
import struct
import sys
from pathlib import Path

import numpy as np

AU_KM = 149597870.7
J2000 = 2451545.0
START_JD, END_JD = 2378282.5, 2597790.5  # 1799-06-01, 2400-05-29

# id: (target, center, km per step, dropped terms may add up to this many km)
BODIES = {
    1: (1, 0, 0.1, 0.1), 2: (2, 0, 0.1, 0.1), 3: (3, 0, 0.002, 0.002), 4: (4, 0, 0.2, 0.2), 5: (5, 0, 0.5, 0.5),
    6: (6, 0, 1.0, 1.0), 7: (7, 0, 2.0, 2.0), 8: (8, 0, 2.0, 2.0), 9: (9, 0, 2.0, 2.0), 10: (10, 0, 0.1, 0.1),
    301: (301, 3, 0.002, 0.002),
}


def read_spk(path):
    """Returns {(target, center): (mid, radius, coefficients[records, axis, degree])} for type 2 segments."""
    raw = np.memmap(path, dtype="u1", mode="r")
    if bytes(raw[:7]) != b"DAF/SPK":
        raise SystemExit("de440.bsp is not an SPK file")
    nd, ni = struct.unpack("<ii", bytes(raw[8:16]))
    fward = struct.unpack("<i", bytes(raw[76:80]))[0]
    ss = nd + (ni + 1) // 2
    words = np.frombuffer(raw, dtype="<f8")
    segments = {}
    record = fward
    while record:
        base = (record - 1) * 128
        nxt, _, count = words[base], words[base + 1], int(words[base + 2])
        for i in range(count):
            summary = base + 3 + i * ss
            ints = np.frombuffer(raw, dtype="<i4", count=ni, offset=(summary + nd) * 8)
            target, center, _, kind, first, last = [int(x) for x in ints[:6]]
            if kind != 2:
                continue
            init, intlen, rsize, n = words[last - 4:last]
            rsize, n = int(rsize), int(n)
            table = np.asarray(words[first - 1:last - 4]).reshape(n, rsize)
            degree = (rsize - 2) // 3
            segments[(target, center)] = (table[:, 0], table[:, 1], table[:, 2:].reshape(n, 3, degree))
        record = int(nxt)
    return segments


def cut(mid, radius, coef):
    start = (mid - radius) / 86400 + J2000
    end = (mid + radius) / 86400 + J2000
    keep = (end > START_JD) & (start < END_JD)
    return start[keep][0], radius[0] * 2 / 86400, coef[keep]


def chiron(path):
    """Fits the Horizons vectors with Chebyshev series, 64 days at a time."""
    rows = re.findall(r"^(\d+\.\d+), A\.D\.[^,]*,\s*(\S+),\s*(\S+),\s*(\S+),", path.read_text(), re.M)
    data = np.array(rows, dtype=float)
    days, degree = 64.0, 9
    n = int((data[-1, 0] - data[0, 0]) // days)
    coef = np.zeros((n, 3, degree))
    per = int(days / 4)
    worst = 0.0
    for i in range(n):
        part = data[i * per : (i + 1) * per + 1]
        x = (part[:, 0] - part[0, 0]) / days * 2 - 1
        for axis in range(3):
            c = np.polynomial.chebyshev.chebfit(x, part[:, axis + 1] * AU_KM, degree - 1)
            coef[i, axis] = c
            worst = max(worst, np.abs(np.polynomial.chebyshev.chebval(x, c) - part[:, axis + 1] * AU_KM).max())
    print(f"Chiron: fit error up to {worst:.3f} km")
    return data[0, 0], days, coef


def pack(body_id, start, days, coef, step, drop):
    worst = np.abs(coef).max(axis=(0, 1))
    keep = len(worst)
    while keep > 2 and worst[keep - 1:].sum() < drop:
        keep -= 1
    q = np.rint(coef[:, :, :keep] / step).astype(np.int64)
    limits = np.abs(q).max(axis=0)
    # One extra bit for the sign; 0 means every value of that coefficient is zero.
    widths = np.where(limits > 0, np.ceil(np.log2(limits + 1)).astype(int) + 1, 0)
    bits = bytearray()
    flat_widths = widths.reshape(-1)
    n_segments = len(q)
    seg_bits = int(flat_widths.sum())
    out = np.zeros(n_segments * seg_bits, dtype=np.uint8)
    position = 0
    flat = q.reshape(n_segments, -1)
    for k, w in enumerate(flat_widths):
        w = int(w)
        if w == 0:
            continue
        values = flat[:, k] & ((1 << w) - 1)
        for b in range(w):
            out[np.arange(n_segments) * seg_bits + position + b] = (values >> b) & 1
        position += w
    data = np.packbits(out, bitorder="little").tobytes()
    head = struct.pack("<IIII", body_id, n_segments, keep, 0) + struct.pack("<ddd", start, days, step)
    print(f"{body_id:4}: {n_segments:6} segments of {days:4.0f} d, {keep:2} of {coef.shape[2]:2} terms, {len(data) / 1e6:6.2f} MB")
    return head, bytes(int(x) for x in flat_widths), data


def delta_t(folder):
    """One value per year: JPL's up to the present, then a parabola.

    JPL keeps its last prediction constant for all later dates, so it has no forecast of its own. The
    Earth keeps slowing down, so after JPL's data the value follows the recent trend bent by the
    long-term tidal parabola of 32.5 s per century squared (Stephenson, Morrison, Hohenkerk 2016).
    """
    text = (folder / "dt_horizons.txt").read_text()
    rows = re.findall(r"^\s*(\d{4})-Jan-01 00:00, , ,\s*(-?[\d.]+),", text.split("$$SOE")[1], re.M)
    known = {int(y): float(v) for y, v in rows}
    last = max(y for y, v in known.items() if y <= 2026)
    trend = (known[last] - known[last - 10]) / 10
    values = [known[1800] + (known[1800] - known[1801])]
    for year in range(1800, 2402):
        d = year - last
        values.append(known[year] if d <= 0 else known[last] + trend * d + 32.5e-4 * d * d)
    print(f"Delta T: {last}: {known[last]:.2f} s, trend {trend:.3f} s/yr, 2100: {values[301]:.1f} s, 2400: {values[601]:.1f} s")
    return 1799, [round(v, 3) for v in values]


def main():
    folder = Path(sys.argv[1])
    root = Path(__file__).resolve().parent.parent
    spk = read_spk(folder / "de440.bsp")
    parts = []
    for body_id, (target, center, step, drop) in BODIES.items():
        parts.append(pack(body_id, *cut(*spk[(target, center)]), step, drop))
    parts.append(pack(2060, *chiron(folder / "chiron.txt"), 1.0, 1.0))

    head_size = 4 + sum(len(h) + len(w) for h, w, _ in parts)
    output = bytearray(struct.pack("<I", len(parts)))
    offset = head_size
    for head, widths, data in parts:
        output += head[:12] + struct.pack("<I", offset) + head[16:] + widths
        offset += len(data)
    for _, _, data in parts:
        output += data
    (root / "core/data/ephem.bin").write_bytes(output)
    print(f"ephem.bin: {len(output) / 1e6:.2f} MB")

    first, values = delta_t(folder)
    lines = ["//! Generated by tools/build_ephem.py. TT - UT in seconds, one value per 1 January.", f"pub const FIRST_YEAR: i32 = {first};", f"pub static DELTA_T: [f32; {len(values)}] = ["]
    for i in range(0, len(values), 10):
        lines.append("    " + ", ".join(f"{v:.3f}" for v in values[i : i + 10]) + ",")
    lines.append("];\n")
    (root / "core/src/deltat_table.rs").write_text("\n".join(lines), encoding="utf8")


main()
