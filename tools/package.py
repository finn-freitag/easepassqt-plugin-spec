#!/usr/bin/env python3
"""Create an .epplugin package (docs/package-format.md) and print its SHA-256.

    package.py --manifest manifest.json --wasm plugin.wasm [--icon icon.png] --out-dir dist

The archive is deterministic: fixed entry order and timestamps, so the same input bytes
always give the same package hash.
"""
import argparse
import hashlib
import json
import sys
import zipfile
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--wasm", type=Path, required=True)
    parser.add_argument("--icon", type=Path)
    parser.add_argument("--out-dir", type=Path, required=True)
    args = parser.parse_args()

    plugin_id = json.loads(args.manifest.read_text(encoding="utf-8"))["id"]
    entries = [("manifest.json", args.manifest), ("plugin.wasm", args.wasm)]
    if args.icon:
        entries.append(("icon.png", args.icon))

    args.out_dir.mkdir(parents=True, exist_ok=True)
    out = args.out_dir / f"{plugin_id}.epplugin"
    with zipfile.ZipFile(out, "w") as archive:
        for name, path in entries:
            info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o644 << 16
            archive.writestr(info, path.read_bytes(), compresslevel=9)

    print(f"{hashlib.sha256(out.read_bytes()).hexdigest()}  {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
