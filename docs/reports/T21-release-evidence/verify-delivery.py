import argparse
import hashlib
import json
from pathlib import Path
from zipfile import ZipFile

ROOT = Path(__file__).resolve().parents[3]
EVIDENCE = Path(__file__).resolve().parent
TOOLING = ROOT / "docs/tooling/evidence/T21"
TEMP = ROOT / "Temp/T21-release"
BASELINE = "a545d0c294eb85276abc5556f297434806ea1def598192ee664593550c579a3c"
CANDIDATE = "76ac4cb51fe8ff1a67cd697540e483f50759e4fd67dce00636781481d492f11e"
ARCHIVE = "a4c2e0650cc2dc4a27a3433a3a811b03890d99f8b229c03672bbc6f2f9f5b197"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def load(path):
    return json.loads(path.read_bytes())


def verify(refresh):
    archive = TOOLING / "candidate-snapshot.zip"
    snapshot = load(TOOLING / "snapshot.json")
    manifest = load(TOOLING / "manifest.json")
    assert sha(archive.read_bytes()) == snapshot["sha256"] == ARCHIVE
    assert archive.stat().st_size == snapshot["bytes"] == 771423
    binary = "target/release/lfz.exe"
    candidate = (ROOT / binary).read_bytes()
    assert len(candidate) == 1117184 and sha(candidate) == CANDIDATE
    expected = {p.relative_to(ROOT).as_posix() for p in (ROOT / "src").rglob("*") if p.is_file()}
    expected |= {p.relative_to(ROOT).as_posix() for p in (ROOT / "tests").rglob("*.rs")}
    expected |= {"Cargo.toml", "Cargo.lock"}
    assert set(manifest["source_hashes"]) == expected
    with ZipFile(archive) as z:
        assert set(z.namelist()) == expected | {binary}
        assert len(z.namelist()) == snapshot["members"] == 42
        for name in z.namelist():
            data = z.read(name)
            assert data == (ROOT / name).read_bytes(), name
            assert sha(data) == (CANDIDATE if name == binary else manifest["source_hashes"][name]), name
    for name, digest in manifest["evidence"].items():
        assert sha((TOOLING / name).read_bytes()) == digest, name

    guide = load(ROOT / "docs/reports/T21-guide-evidence/runs.json")
    assert guide["binary_sha256"] == guide["final_binary_sha256"] == CANDIDATE
    assert guide["docs"] == guide["final_docs"]
    for name, digest in guide["docs"].items():
        assert sha((ROOT / name).read_bytes()) == digest, name

    independent_path = ROOT / "docs/reports/T21-independent-evidence/manifest.json"
    independent_sha = sha(independent_path.read_bytes())
    assert independent_sha == "9c8d5a1f47cd186cf6ff8cef6d1f7c616986e4129846f5ac393d2bec8de16795"
    independent = load(independent_path)["files"]
    assert len(independent) == 72
    bound = dict(guide["docs"])
    for name, evidence in independent.items():
        data = (ROOT / name).read_bytes()
        assert len(data) == evidence["bytes"] and sha(data) == evidence["sha256"], name
        bound[name] = evidence["sha256"]
    bound[independent_path.relative_to(ROOT).as_posix()] = independent_sha
    bound.update({f"docs/tooling/evidence/T21/{name}": digest for name, digest in manifest["evidence"].items()})

    dist = ROOT / "dist/lfz.exe"
    previous = TEMP / "lfz-T20-a545d0.exe"
    if refresh:
        TEMP.mkdir(parents=True, exist_ok=True)
        current = dist.read_bytes()
        assert sha(current) in {BASELINE, CANDIDATE}
        if sha(current) == BASELINE and not previous.exists():
            previous.write_bytes(current)
        assert sha(previous.read_bytes()) == BASELINE and previous.stat().st_size == 1100800
        if current != candidate:
            dist.write_bytes(candidate)
    assert dist.read_bytes() == candidate
    receipt = {
        "binary": {"sha256": CANDIDATE, "bytes": len(candidate), "dist": dist.relative_to(ROOT).as_posix()},
        "candidate_archive": {"path": archive.relative_to(ROOT).as_posix(), **snapshot},
        "previous_dist": {"path": previous.relative_to(ROOT).as_posix(), "sha256": BASELINE},
        "source_and_rust_test_files": len(expected),
        "tooling_evidence_files": len(manifest["evidence"]),
        "independent_evidence_files": len(independent),
        "byte_bound_git_paths": dict(sorted(bound.items())),
    }
    (EVIDENCE / "archive-verification.json").write_text(
        json.dumps(receipt, indent=2) + "\n", encoding="utf-8", newline="\n"
    )
    print(json.dumps({key: value for key, value in receipt.items() if key != "byte_bound_git_paths"}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--refresh-dist", action="store_true")
    verify(parser.parse_args().refresh_dist)
