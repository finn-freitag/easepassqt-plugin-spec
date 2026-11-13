#!/usr/bin/env python3
"""Compute the types and permissions of a plugin component the way a host must do it.

Reference implementation of docs/permissions.md: reads the imports and exports of a Wasm
component with `wasm-tools component wit` (the component is never instantiated) and maps
them through spec/permissions.json.

    check_component.py plugin.wasm                       print types/permissions as JSON
    check_component.py plugin.wasm --expected e.json     must match the expected values
    check_component.py plugin.wasm --manifest m.json     manifest must agree
    check_component.py plugin.wasm --expect-rejected     must be rejected

Exit code 0 = as expected, 1 = not as expected.
"""
import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ITEM = re.compile(r"^\s*(import|export)\s+([^;{]+?)\s*[;{]")
NAME = re.compile(r"^([a-z0-9-]+:[a-z0-9-]+/[a-z0-9-]+)@(\d+)\.(\d+)\.(\d+)$")


def world_items(component: Path):
    """Return (imports, exports) of the component's root world as written in WIT."""
    wit = subprocess.run(
        ["wasm-tools", "component", "wit", str(component)],
        check=True, capture_output=True, text=True,
    ).stdout
    imports, exports = [], []
    in_world = False
    for line in wit.splitlines():
        if not in_world:
            in_world = line.startswith("world ")
            continue
        if line.startswith("}"):
            break
        match = ITEM.match(line)
        if match:
            (imports if match.group(1) == "import" else exports).append(match.group(2))
    return imports, exports


def inspect(component: Path, table: dict) -> dict:
    """Map imports/exports to permissions/types. `rejected` lists why a host must refuse."""
    api = table["api_version"]
    neutral = set(table["neutral_imports"])
    permission_of = {p["import"]: p["id"] for p in table["permissions"]}
    type_of = {t["export"]: t["id"] for t in table["types"]}
    imports, exports = world_items(component)

    def base_name(item: str, kind: str, rejected: list):
        match = NAME.match(item)
        if not match:
            rejected.append(f"{kind} '{item}' is not a versioned interface")
            return None
        name, major, minor = match.group(1), int(match.group(2)), int(match.group(3))
        if name.startswith("wasi:"):
            compatible = (major, minor) == (0, 2)
        else:
            compatible = major == api
        if not compatible:
            rejected.append(f"{kind} '{item}' has an unsupported version")
            return None
        return name

    rejected, permissions, types = [], [], []
    for item in imports:
        name = base_name(item, "import", rejected)
        if name is None:
            continue
        if name in permission_of:
            permissions.append(permission_of[name])
        elif name not in neutral:
            rejected.append(f"import '{item}' is not on the allow-list")
    for item in exports:
        name = base_name(item, "export", rejected)
        if name is None:
            continue
        if name in type_of:
            types.append(type_of[name])
        else:
            rejected.append(f"export '{item}' is not a known plugin interface")
    if not types and not rejected:
        rejected.append("the component exports no plugin interface")

    order_p = [p["id"] for p in table["permissions"]]
    order_t = [t["id"] for t in table["types"]]
    return {
        "types": sorted(set(types), key=order_t.index),
        "permissions": sorted(set(permissions), key=order_p.index),
        "rejected": rejected,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("component", type=Path)
    parser.add_argument("--expected", type=Path, help="JSON with expected 'types' and 'permissions'")
    parser.add_argument("--manifest", type=Path, help="manifest.json that must agree with the component")
    parser.add_argument("--expect-rejected", action="store_true")
    args = parser.parse_args()

    table = json.loads((ROOT / "spec" / "permissions.json").read_text(encoding="utf-8"))
    result = inspect(args.component, table)
    print(json.dumps(result, indent=2))

    if args.expect_rejected:
        if result["rejected"]:
            return 0
        print(f"FAIL {args.component}: expected a rejection", file=sys.stderr)
        return 1

    ok = True
    if result["rejected"]:
        ok = False
        print(f"FAIL {args.component}: rejected", file=sys.stderr)
    if args.expected:
        expected = json.loads(args.expected.read_text(encoding="utf-8"))
        for key in ("types", "permissions"):
            if sorted(expected[key]) != sorted(result[key]):
                ok = False
                print(f"FAIL {args.component}: {key} {result[key]} != expected {expected[key]}", file=sys.stderr)
    if args.manifest:
        manifest = json.loads(args.manifest.read_text(encoding="utf-8"))
        if manifest["api_version"] != table["api_version"]:
            ok = False
            print(f"FAIL {args.manifest}: api_version {manifest['api_version']}", file=sys.stderr)
        declared = manifest.get("permissions")
        if declared is not None and sorted(declared) != sorted(result["permissions"]):
            ok = False
            print(f"FAIL {args.manifest}: permissions {declared} != computed {result['permissions']}", file=sys.stderr)
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
