#!/usr/bin/env python3
"""Count PTX instruction categories per kernel. Usage: ptx_count.py <file.ptx> -> JSON on stdout."""
import json
import re
import sys

CATS = [
    ("ld.global", "ld.global"), ("st.global", "st.global"),
    ("ld.shared", "ld.shared"), ("st.shared", "st.shared"),
    ("ld.param", "ld.param"), ("ld.local", "ld.local"), ("st.local", "st.local"),
    ("fma", "fma"), ("mad", "mad"), ("mul.hi", "mul.hi"), ("mul.wide", "mul.wide"),
    ("setp", "setp"), ("selp", "selp"), ("bra", "bra"),
    ("atom", "atom"), ("red", "red"), ("shfl", "shfl"), ("vote", "vote"),
    ("bar", "bar"), ("call", "call"), ("popc", "popc"), ("clz", "clz"),
]


def main(path):
    kernels = {}
    cur = None
    depth = 0
    for raw in open(path, encoding="utf-8", errors="replace"):
        line = raw.split("//")[0].strip()
        if not line:
            continue
        m = re.match(r"\.(?:visible\s+)?\.entry\s+(\w+)", line)
        if m:
            cur = {"total": 0}
            kernels[m.group(1)] = cur
            depth = 0
            continue
        if cur is None:
            continue
        depth += line.count("{") - line.count("}")
        if "{" in line:
            cur["_body"] = True
        if not cur.get("_body"):
            continue                      # parameter list before the body
        if line in ("{", "}") or line.startswith((".", "$", "{", "}")) or line.endswith(":"):
            if depth <= 0 and "}" in line:
                cur = None
            continue
        line = re.sub(r"^@!?%p\d+\s+", "", line)      # drop the predicate
        op = line.split()[0].rstrip(";")
        cur["total"] += 1
        for key, prefix in CATS:
            if op.startswith(prefix):
                cur[key] = cur.get(key, 0) + 1
    totals = {}
    for k in kernels.values():
        k.pop("_body", None)
        for name, v in k.items():
            totals[name] = totals.get(name, 0) + v
    json.dump({"kernels": kernels, "totals": totals}, sys.stdout)


if __name__ == "__main__":
    main(sys.argv[1])
