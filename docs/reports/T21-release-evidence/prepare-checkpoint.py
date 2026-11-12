import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
from zipfile import ZipFile

ROOT = Path(__file__).resolve().parents[3]
EVIDENCE = Path(__file__).resolve().parent
RECEIPT = EVIDENCE / "archive-verification.json"
STAGE = ROOT / "Temp/T21-release/stage"
BASE = "881d3f92c97f4adc8dc8c06f067adb50a60a7cd2"
REPORT = "docs/reports/"
PRODUCT_TESTS = {"tests/cli_t21.rs", "tests/t21_cli_fmt.py"}
PUBLIC_TEST_SHA = "d5bd25dd0d809c6c4647d614525010fabaa5d2bdf08fe4f51455411bbdff6deb"
MESSAGES = [
    "docs(spec): define position-preserving whitespace formatting",
    "feat(fmt): normalize top-level code whitespace without rewriting source",
    "test(fmt): verify byte fidelity and legacy CLI compatibility",
    "docs(guide): document safe formatter invocation",
    "chore(checkpoint): archive the verified T21 candidate",
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
    if path.startswith("docs/spec/") or path in {
        ".gitattributes", ".opencode/team/REQUIREMENTS.md", REPORT + "T21-contract.md",
    }:
        return 1
    if path in {"src/cli.rs", "src/formatter.rs", "src/lexer.rs", "src/lib.rs"}:
        return 2
    if path in PRODUCT_TESTS or path.startswith((
        REPORT + "T21-blackbox", REPORT + "T21-corpus", REPORT + "T21-independent",
        REPORT + "T21-tooling", "docs/tooling/evidence/T21/",
    )):
        return 3
    if path in {"docs/guide/README.md", "docs/guide/formatting.md"} or path.startswith(REPORT + "T21-guide"):
        return 4
    if path in {"README.md", REPORT + "delivery-checklist.md"} or path.startswith((
        ".opencode/team/", REPORT + "T21-release-evidence/",
    )):
        return 5
    raise ValueError(f"Unexpected checkpoint path: {path}")


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

    receipt = json.loads(RECEIPT.read_bytes())
    archive_path = ROOT / receipt["candidate_archive"]["path"]
    assert sha(archive_path.read_bytes()) == receipt["candidate_archive"]["sha256"]
    with ZipFile(archive_path) as archive:
        frozen = {path: archive.read(path) for path in archive.namelist() if path.startswith("src/")}
        frozen["tests/cli_t21.rs"] = archive.read("tests/cli_t21.rs")
    assert len(frozen) == 26
    public_test = ROOT / "tests/t21_cli_fmt.py"
    frozen["tests/t21_cli_fmt.py"] = public_test.read_bytes()
    assert sha(frozen["tests/t21_cli_fmt.py"]) == PUBLIC_TEST_SHA
    for path in sorted(paths):
        if path.startswith("src/") or path in PRODUCT_TESTS:
            assert path in frozen and (ROOT / path).read_bytes() == frozen[path], path
        if path.startswith("tests/"):
            assert path in PRODUCT_TESTS, path

    bound = receipt["byte_bound_git_paths"]
    assert len(bound) == 114
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
        "date": "2026-11-13",
        "baseline": BASE,
        "status": "T21 candidate checkpoint; R304 OPEN; no formal release tag",
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
