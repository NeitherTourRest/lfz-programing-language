import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[3]
EVIDENCE = Path(__file__).resolve().parent
STAGE = ROOT / "Temp/T22-release/stage"
BASE = "71caa6d82a0e958a15b66354e3bebe3e4748a086"
REPORT = "docs/reports/"
CANDIDATE = "76ac4cb51fe8ff1a67cd697540e483f50759e4fd67dce00636781481d492f11e"
MANIFESTS = (
    ("docs/tooling/evidence/T22", "sha256", False),
    (REPORT + "T22-independent-evidence", "files", True),
    (REPORT + "T22-pipe-evidence", "files", False),
)
MESSAGES = [
    "test(repl): verify Windows console Chinese rendering",
    "docs(repl): document Windows console and redirection behavior",
    "chore(checkpoint): record T22 console validation",
]
PROTECTED = {
    ".opencode/team/DECISIONS.md": (
        r"(?ms)^### \[2026-09-27 23:55\].*?(?=^### \[2026-10-24)", 28
    ),
    ".opencode/team/PROJECT_STATE.md": (
        r"(?m)^- \*\*⚠️ 崩溃转储（2026-09-27 事故，ADR 已记）\*\*：[^\n]*\n", 1
    ),
    ".opencode/team/agents/team-lead/STATUS.md": (
        r"(?m)^- \*\*🚨 崩溃压测会吃磁盘（2026-09-27 事故，已立 ADR）\*\*：[^\n]*\n", 1
    ),
}


def git(*args, input=None):
    return subprocess.run(["git", *args], cwd=ROOT, check=True, capture_output=True, input=input).stdout


def sha(data):
    return hashlib.sha256(data).hexdigest()


def group(path):
    if path == ".gitattributes" or path.startswith((
        REPORT + "T22-independent", REPORT + "T22-pipe-evidence/", "docs/tooling/evidence/T22/",
    )) or path == REPORT + "T22-tooling.md":
        return 1
    if path in {"README.md", "docs/guide/repl.md"}:
        return 2
    if path.startswith((".opencode/team/", REPORT + "T22-release-evidence/")):
        return 3
    raise ValueError(f"Unexpected checkpoint path: {path}")


def collect_bound_paths():
    dist = ROOT / "dist/lfz.exe"
    binary = dist.read_bytes()
    assert len(binary) == 1117184 and sha(binary) == CANDIDATE
    bound = {}
    for directory, key, root_relative in MANIFESTS:
        manifest_path = f"{directory}/manifest.json"
        manifest_data = (ROOT / manifest_path).read_bytes()
        manifest = json.loads(manifest_data)
        if "binary" in manifest:
            assert manifest["binary"]["sha256"] == CANDIDATE
        for name, evidence in manifest[key].items():
            path = name if root_relative else f"{directory}/{name}"
            data = (ROOT / path).read_bytes()
            if isinstance(evidence, str):
                digest = evidence
            else:
                assert len(data) == evidence["bytes"], path
                digest = evidence["sha256"]
            assert sha(data) == digest, path
            bound[path] = digest
        bound[manifest_path] = sha(manifest_data)
    return dict(sorted(bound.items()))


def prepare(write):
    assert git("rev-parse", "HEAD").decode().strip() == BASE
    assert not git("diff", "--cached", "--name-only")
    assert not git("diff", BASE, "--", "Cargo.toml", "Cargo.lock",
                   "docs/validation/agent-benchmark-v2", "docs/reports/agent-benchmark-v2",
                   "docs/validation/agent-benchmark-v3", "docs/reports/T20-agent-v3-evidence",
                   ".opencode/skills/lfz-programming", "docs/guide/ai/README.md")
    listing = git("diff", BASE, "--name-only", "-z") + git("ls-files", "--others", "--exclude-standard", "-z")
    paths = set(filter(None, listing.decode().split("\0")))
    paths.add((EVIDENCE / "commit-groups.json").relative_to(ROOT).as_posix())

    frozen = {}
    bound = collect_bound_paths()
    raw_paths = ("\n".join(bound) + "\n").encode()
    attributes = git("check-attr", "text", "--stdin", input=raw_paths).decode().splitlines()
    assert attributes == [f"{path}: text: unset" for path in bound]
    assert git("hash-object", "--stdin-paths", input=raw_paths) == git(
        "hash-object", "--no-filters", "--stdin-paths", input=raw_paths
    )
    for path, digest in bound.items():
        assert sha((ROOT / path).read_bytes()) == digest, path
        assert path in paths, path
    for path in sorted(paths):
        assert path.endswith("commit-groups.json") or (ROOT / path).is_file(), path

    for path, (pattern, lines) in PROTECTED.items():
        original = (ROOT / path).read_text(encoding="utf-8")
        matches = re.findall(pattern, original)
        assert len(matches) == 1 and len(matches[0].splitlines()) == lines, path
        frozen[path] = re.sub(pattern, "", original).encode("utf-8")
        assert path in paths

    snapshots = []
    for path in sorted(frozen.keys() & paths):
        data = frozen[path]
        output = STAGE / path.replace("/", "__")
        if write:
            STAGE.mkdir(parents=True, exist_ok=True)
            output.write_bytes(data)
        record = {"path": path, "staged_copy": output.relative_to(ROOT).as_posix(), "copy_sha256": sha(data)}
        if path in PROTECTED:
            record.update(removed_matches=1, removed_lines=PROTECTED[path][1])
        snapshots.append(record)
    snapshot_paths = {entry["path"] for entry in snapshots}
    grouped = {number: [] for number in range(1, len(MESSAGES) + 1)}
    for path in sorted(paths):
        grouped[group(path)].append(path)
    groups = [{
        "sequence": number,
        "message": MESSAGES[number - 1],
        "direct_stage_paths": [path for path in selected if path not in snapshot_paths],
        "staged_snapshot_paths": [path for path in selected if path in snapshot_paths],
    } for number, selected in grouped.items()]
    plan = {
        "date": "2026-11-14",
        "baseline": BASE,
        "status": "T22 Windows console evidence for T21 candidate; R304 OPEN",
        "snapshots": snapshots,
        "byte_bound_git_paths": bound,
        "groups": groups,
    }
    if write:
        (EVIDENCE / "commit-groups.json").write_text(
            json.dumps(plan, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n"
        )
    print(json.dumps({
        "paths": len(paths),
        "groups": [[g["sequence"], len(g["direct_stage_paths"]), len(g["staged_snapshot_paths"])] for g in groups],
        "snapshots": len(snapshots),
        "byte_bound_paths": len(bound),
        "check_only": not write,
    }))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--check-only", action="store_true")
    prepare(not parser.parse_args().check_only)
