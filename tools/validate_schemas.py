#!/usr/bin/env python3
"""Validate the JSON schemas and the sample files.

Every file in samples/<kind>/valid must pass, every file in samples/<kind>/invalid must
fail, and every conformance manifest must pass. Exit code 1 on any mismatch.
"""
import json
import sys
from pathlib import Path

from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parent.parent


def load(path: Path):
    with path.open(encoding="utf-8") as f:
        return json.load(f)


def main() -> int:
    failures = 0
    checked = 0
    for kind in ("manifest", "catalog"):
        schema = load(ROOT / "schemas" / f"{kind}.schema.json")
        Draft202012Validator.check_schema(schema)
        validator = Draft202012Validator(schema)

        expectations = [(p, True) for p in sorted((ROOT / "samples" / kind / "valid").glob("*.json"))]
        expectations += [(p, False) for p in sorted((ROOT / "samples" / kind / "invalid").glob("*.json"))]
        if kind == "manifest":
            expectations += [(p, True) for p in sorted((ROOT / "conformance").glob("*/manifest.json"))]
        if not any(ok for _, ok in expectations) or not any(not ok for _, ok in expectations):
            print(f"FAIL {kind}: need at least one valid and one invalid sample")
            failures += 1

        for path, should_pass in expectations:
            errors = list(validator.iter_errors(load(path)))
            checked += 1
            rel = path.relative_to(ROOT)
            if should_pass and errors:
                failures += 1
                print(f"FAIL {rel}: expected valid, got: {errors[0].message}")
            elif not should_pass and not errors:
                failures += 1
                print(f"FAIL {rel}: expected invalid, but it validates")
            else:
                print(f"ok   {rel} ({'valid' if should_pass else 'rejected'})")

    # The permission table must be valid JSON with unique ids.
    table = load(ROOT / "spec" / "permissions.json")
    for key in ("permissions", "types"):
        ids = [entry["id"] for entry in table[key]]
        if len(ids) != len(set(ids)):
            failures += 1
            print(f"FAIL spec/permissions.json: duplicate id in {key}")
    manifest_schema = load(ROOT / "schemas" / "manifest.schema.json")
    schema_permissions = set(manifest_schema["properties"]["permissions"]["items"]["enum"])
    if schema_permissions != {p["id"] for p in table["permissions"]}:
        failures += 1
        print("FAIL manifest.schema.json: permission enum differs from spec/permissions.json")

    print(f"{checked} files checked, {failures} failure(s)")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
