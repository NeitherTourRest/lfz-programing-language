import argparse
import hashlib
import json
from pathlib import Path
from zipfile import ZipFile

ROOT = Path(__file__).resolve().parents[3]
EVIDENCE = Path(__file__).resolve().parent
TOOLING = ROOT / "docs/tooling/evidence/T20"
V3 = ROOT / "docs/validation/agent-benchmark-v3/v3-t20-r01"
REPORTS = ROOT / "docs/reports"
TEMP = ROOT / "Temp/T20-release"
CANDIDATE = "a545d0c294eb85276abc5556f297434806ea1def598192ee664593550c579a3c"
T19 = "5d2219f88db8b3196cac259941fc0a8a3b489ef03519bc7e20415f8edf6388e5"
V3_MANIFEST = "793caa2c2a006ab6a843b58513040f52ef164ac0ce039f760bd06baa798763b9"
V3_ARCHIVE = "90932d39b7dca058b4d64acaa5d6af78d33fb6d050c57451c4b83fe2080a0b08"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def load(path):
    return json.loads(path.read_bytes())


def verify():
    snapshot_path = TOOLING / "candidate-snapshot.zip"
    snapshot = load(TOOLING / "snapshot.json")
    assert sha(snapshot_path.read_bytes()) == snapshot["sha256"]
    assert snapshot_path.stat().st_size == snapshot["bytes"]
    assert snapshot["sha256"] == "31f34b432db98ca8ac139b1b5f1a8c8a08d379cd69769e0fb52f593504fb6ff0"
    expected_src = {path.relative_to(ROOT).as_posix() for path in (ROOT / "src").rglob("*") if path.is_file()}
    expected_tests = {path.relative_to(ROOT).as_posix() for path in (ROOT / "tests").rglob("*.rs")}
    binary_path = "target/release/lfz.exe"
    with ZipFile(snapshot_path) as archive:
        members = archive.namelist()
        assert len(members) == snapshot["members"] == 40
        assert set(members) == expected_src | expected_tests | {"Cargo.toml", "Cargo.lock", binary_path}
        candidate_files = {}
        for name in members:
            data = archive.read(name)
            assert data == (ROOT / name).read_bytes(), name
            candidate_files[name] = {"bytes": len(data), "sha256": sha(data)}
    candidate = (ROOT / binary_path).read_bytes()
    assert len(candidate) == 1100800 and sha(candidate) == CANDIDATE

    manifest_path = V3 / "manifest.frozen.json"
    manifest = load(manifest_path)
    assert sha(manifest_path.read_bytes()) == V3_MANIFEST
    assert manifest["status"] == "FROZEN" and manifest["final_binary"]["sha256"] == CANDIDATE
    zip_path = V3 / "frozen-package.zip"
    assert sha(zip_path.read_bytes()) == V3_ARCHIVE
    ai_evidence = load(REPORTS / "T20-ai-repl-evidence/manifest.json")
    assert ai_evidence["binary"]["sha256"] == CANDIDATE
    assert ai_evidence["documents"] == manifest["final_ai_docs"]
    human = load(REPORTS / "T20-independent-evidence/human-guide-original.json")
    assert human["binary_sha256"] == CANDIDATE
    human_docs = {name.replace(chr(92), "/"): digest for name, digest in human["docs"].items()}
    assert len(human_docs) == 6
    bound_paths = {}
    with ZipFile(zip_path) as archive:
        expected = dict(manifest["files"])
        expected["manifest.frozen.json"] = V3_MANIFEST
        expected["independent/v3-controls-receipt.json"] = manifest["independent_review"]["sha256"]
        expected["runtime/lfz.exe"] = CANDIDATE
        expected.update({f"ai-docs/{name}": digest for name, digest in manifest["final_ai_docs"].items()})
        assert len(archive.namelist()) == len(expected) and set(archive.namelist()) == set(expected)
        for name, digest in expected.items():
            data = archive.read(name)
            assert sha(data) == digest, name
            if name.startswith("ai-docs/"):
                path = ROOT / name.removeprefix("ai-docs/")
            elif name.startswith("independent/"):
                path = ROOT / manifest["independent_review"]["path"]
            elif name.startswith("runtime/"):
                path = ROOT / binary_path
            else:
                path = V3 / name
            assert path.read_bytes() == data, path
            if path != ROOT / binary_path:
                bound_paths[path.relative_to(ROOT).as_posix()] = digest
    for name, digest in human_docs.items():
        assert sha((ROOT / name).read_bytes()) == digest, name
        bound_paths[name] = digest
    bound_paths[zip_path.relative_to(ROOT).as_posix()] = V3_ARCHIVE

    student_root = REPORTS / "T20-agent-v3-evidence"
    student_manifest = student_root / "manifest.json"
    students = load(student_manifest)["files"]
    for name, evidence in students.items():
        data = (ROOT / name).read_bytes()
        assert len(data) == evidence["bytes"] and sha(data) == evidence["sha256"], name
        bound_paths[name] = evidence["sha256"]
    summary = load(student_root / "summary.json")
    assert summary["manifest_sha256"] == V3_MANIFEST and summary["binary_sha256"] == CANDIDATE
    assert summary["certified"] == "4/4" and len(summary["tasks"]) == 4
    student_manifest_sha = sha(student_manifest.read_bytes())
    bound_paths[student_manifest.relative_to(ROOT).as_posix()] = student_manifest_sha
    return candidate, {
        "binary": candidate_files[binary_path],
        "candidate_archive": {"path": snapshot_path.relative_to(ROOT).as_posix(), **snapshot},
        "candidate_archive_members": candidate_files,
        "v3_archive": {"path": zip_path.relative_to(ROOT).as_posix(), "members": len(expected), "sha256": V3_ARCHIVE},
        "v3_frozen_manifest_sha256": V3_MANIFEST,
        "v3_student_evidence": {"files": len(students), "manifest_sha256": student_manifest_sha},
        "byte_bound_git_paths": dict(sorted(bound_paths.items())),
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--refresh-dist", action="store_true")
    args = parser.parse_args()
    candidate, receipt = verify()
    dist = ROOT / "dist/lfz.exe"
    previous = TEMP / "lfz-T19-5d2219.exe"
    if args.refresh_dist:
        TEMP.mkdir(parents=True, exist_ok=True)
        current = dist.read_bytes()
        if sha(current) == T19 and not previous.exists():
            previous.write_bytes(current)
        assert sha(previous.read_bytes()) == T19
        assert sha(current) in {T19, CANDIDATE}
        if current != candidate:
            dist.write_bytes(candidate)
    assert dist.read_bytes() == candidate
    receipt["local_dist"] = "dist/lfz.exe"
    receipt["previous_dist_sha256"] = T19
    (EVIDENCE / "archive-verification.json").write_text(
        json.dumps(receipt, indent=2) + "\n", encoding="utf-8", newline="\n"
    )
    print(json.dumps({key: receipt[key] for key in ("binary", "candidate_archive", "v3_archive")}))


if __name__ == "__main__":
    main()
