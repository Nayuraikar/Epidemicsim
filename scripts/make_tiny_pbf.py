#!/usr/bin/env python3
"""Generate crates/data/tests/data/tiny.osm.pbf (stdlib only, uncompressed blobs).

Usage: python3 -I scripts/make_tiny_pbf.py
Area: lon 77.00..77.02, lat 12.00..12.02 (plus one building outside, lon 78.0).
"""
import struct, pathlib

def varint(n):
    n &= (1 << 64) - 1
    out = bytearray()
    while True:
        b = n & 0x7F
        n >>= 7
        if n:
            out.append(b | 0x80)
        else:
            out.append(b)
            return bytes(out)

def zz(n):
    return (n << 1) ^ (n >> 63)

def tag(f, w):
    return varint((f << 3) | w)

def vfield(f, n):
    return tag(f, 0) + varint(n)

def lfield(f, b):
    return tag(f, 2) + varint(len(b)) + b

def packed(f, vals, signed=False):
    return lfield(f, b"".join(varint(zz(v) if signed else v) for v in vals))

def delta(vals):
    prev, out = 0, []
    for v in vals:
        out.append(v - prev)
        prev = v
    return out

strings = [b""]
def sid(s):
    b = s.encode()
    if b not in strings:
        strings.append(b)
    return strings.index(b)

def e7(deg):
    return round(deg * 1e7)  # granularity 100 nm

nodes = {}   # id -> (lat, lon)
nid = [1000]
def node(lat, lon):
    nid[0] += 1
    nodes[nid[0]] = (e7(lat), e7(lon))
    return nid[0]

ways = []
def way(wid, tags, pts):
    refs = [node(la, lo) for la, lo in pts]
    refs.append(refs[0])
    ks = [sid(k) for k in tags]
    vs = [sid(v) for v in tags.values()]
    ways.append(lfield(3, vfield(1, wid) + packed(2, ks) + packed(3, vs) + packed(8, delta(refs), True)))

def square(lat, lon, d=0.0002):
    return [(lat, lon), (lat, lon + d), (lat + d, lon + d), (lat + d, lon)]

wid = 1
for r in range(3):
    for c in range(4):
        t = {"building": "yes"}
        if (r, c) == (0, 0):
            t = {"building": "apartments", "building:levels": "4"}
        elif (r, c) == (0, 1):
            t = {"building": "residential"}
        way(wid, t, square(12.004 + r * 0.004, 77.004 + c * 0.004))
        wid += 1
way(100, {"office": "company"}, square(12.016, 77.016))
way(101, {"building": "commercial"}, square(12.016, 77.012))
way(102, {"amenity": "school", "building": "school"}, square(12.002, 77.002))
way(103, {"amenity": "college"}, square(12.018, 77.002))
way(104, {"amenity": "hospital", "building": "hospital"}, square(12.010, 77.018))
way(105, {"shop": "supermarket", "building": "retail"}, square(12.012, 77.006))
way(106, {"building": "yes"}, square(12.005, 78.000))  # outside bbox
way(107, {"building": "yes", "shop": "bakery"}, square(12.006, 77.0105))  # tagged shop, not a home

# tagged POI node: marketplace
mk = node(12.0071, 77.0151)
poi = lfield(1, tag(1, 0) + varint(zz(mk)) + packed(2, [sid("amenity")]) + packed(3, [sid("marketplace")]) + tag(8, 0) + varint(zz(nodes[mk][0])) + tag(9, 0) + varint(zz(nodes[mk][1])))
del nodes[mk]

ids = sorted(nodes)
dense = (packed(1, delta(ids), True) + packed(8, delta([nodes[i][0] for i in ids]), True)
         + packed(9, delta([nodes[i][1] for i in ids]), True))

st = b"".join(lfield(1, s) for s in strings)
# groups: dense group, node group, ways group (each its own PrimitiveGroup)
block = (lfield(1, st) + lfield(2, lfield(2, dense)) + lfield(2, poi)
         + lfield(2, b"".join(ways)) + vfield(17, 100))

def blob(kind, payload):
    b = lfield(1, payload) + vfield(2, len(payload))
    h = lfield(1, kind.encode()) + vfield(3, len(b))
    return struct.pack(">I", len(h)) + h + b

header = lfield(4, b"OsmSchema-V0.6") + lfield(4, b"DenseNodes")
out = pathlib.Path(__file__).resolve().parent.parent / "crates/data/tests/data/tiny.osm.pbf"
out.parent.mkdir(parents=True, exist_ok=True)
out.write_bytes(blob("OSMHeader", header) + blob("OSMData", block))
print("wrote", out, out.stat().st_size, "bytes")
