"""Archive the candidate executable and the byte-exact inputs used to compile it."""
import argparse
import hashlib
import json
from pathlib import Path
import zipfile

ROOT = Path(__file__).resolve().parents[2]


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--expected-sha", required=True)
    parser.add_argument("--directory", type=Path, required=True)
    args = parser.parse_args()
    out = args.directory.resolve()
    if not out.is_relative_to(ROOT):
        parser.error("--directory must be inside the project")
    payload = {"lfz.exe": args.binary.read_bytes()}
    assert digest(payload["lfz.exe"]) == args.expected_sha
    inputs = sorted((ROOT / "src").rglob("*.rs")) + [ROOT / "Cargo.toml", ROOT / "Cargo.lock"]
    for path in inputs:
        payload[path.relative_to(ROOT).as_posix()] = path.read_bytes()

    archive = out / "candidate-snapshot.zip"
    manifest = out / "candidate-snapshot.json"
    assert not archive.exists() and not manifest.exists()
    with zipfile.ZipFile(archive, "x", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as writer:
        for name, data in payload.items():
            entry = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            writer.writestr(entry, data, compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)
    with zipfile.ZipFile(archive) as reader:
        assert reader.namelist() == list(payload)
        assert all(reader.read(name) == data for name, data in payload.items())
    record = {"archive_sha256": digest(archive.read_bytes()), "archive_bytes": archive.stat().st_size,
              "entries": {name: {"sha256": digest(data), "bytes": len(data)}
                          for name, data in payload.items()}}
    with manifest.open("x", encoding="utf8", newline="\n") as handle:
        json.dump(record, handle, indent=2)
        handle.write("\n")
    print(record["archive_sha256"])


if __name__ == "__main__":
    main()
