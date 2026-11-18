"""Run the JSON rule router against independently computed expectations."""

import argparse
import hashlib
import json
import os
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
APP = ROOT / "app/jsonroute.lfz"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def plan(rules, kind="build", severity=4):
    return {
        "event": {"kind": kind, "severity": severity},
        "rules": [
            {"kind": rule_kind, "minSeverity": minimum, "target": target}
            for rule_kind, minimum, target in rules
        ],
    }


def expected(request):
    event = request["event"]
    for index, rule in enumerate(request["rules"]):
        if rule["kind"] in ("*", event["kind"]) and event["severity"] >= rule["minSeverity"]:
            return dict(matched=True, index=index, target=rule["target"], checked=index + 1)
    return dict(matched=False, index=None, target=None, checked=len(request["rules"]))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--expected-sha")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    overlap = [("build", 4, "build-oncall"), ("*", 0, "general")]
    invalid_tail = plan(overlap)
    invalid_tail["rules"][1]["minSeverity"] = True
    cases = [
        ("first_overlap", plan(overlap), "stdin", None),
        ("reversed_priority", plan(list(reversed(overlap))), "stdin", None),
        ("second_match", plan([("build", 5, "urgent"), *overlap]), "argv", None),
        ("unicode_target", plan([("构建", 1, "值班组")], "构建", 1), "stdin", None),
        ("unmatched", plan([("test", 1, "testing"), ("build", 5, "urgent")]), "stdin", None),
        ("empty", plan([]), "stdin", None),
        ("invalid_rule_after_match", invalid_tail, "stdin", "rules[1].minSeverity 必须是非负整数"),
        ("missing_field", {"event": {"kind": "build"}, "rules": []}, "stdin", "event 缺少 severity"),
        ("invalid_json", "{broken", "stdin", "ValueError:"),
    ]
    source_sha = sha(APP)
    if args.prepare_only:
        print(json.dumps(dict(prepared=True, cases=len(cases), source_sha256=source_sha, interpreter_processes=0)))
        return

    if not (args.binary and args.expected_sha and args.output):
        parser.error("provide --binary, --expected-sha and --output")
    binary = args.binary.resolve()
    output = args.output.resolve()
    assert binary.is_relative_to(ROOT) and output.is_relative_to(ROOT) and not output.exists()
    assert sha(binary) == args.expected_sha.lower()
    work = ROOT / "Temp/T27-app"
    work.mkdir(parents=True, exist_ok=True)
    output.parent.mkdir(parents=True, exist_ok=True)
    rows = []
    result = dict(binary_sha256=sha(binary), binary_bytes=binary.stat().st_size, source_sha256=source_sha,
                  status="running", rows=rows)

    def save():
        output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf8")

    try:
        for name, request, mode, error in cases:
            payload = request if isinstance(request, str) else json.dumps(request, ensure_ascii=False)
            command = [str(binary), "run", str(APP)]
            data = b""
            if mode == "argv":
                command += ["--", payload]
            else:
                data = (payload + "\n").encode("utf8")
            want = expected(request) if error is None else error
            row = dict(name=name, argv=command, payload=payload, mode=mode, expected=want, passed=False)
            rows.append(row)
            save()
            try:
                proc = subprocess.run(
                    command, input=data, capture_output=True, cwd=ROOT, timeout=10,
                    env=dict(os.environ, TEMP=str(work), TMP=str(work)),
                    creationflags=0x08000000 if os.name == "nt" else 0,
                )
            except subprocess.TimeoutExpired as exc:
                row.update(watchdog=True, stdout_hex=(exc.stdout or b"").hex(), stderr_hex=(exc.stderr or b"").hex())
                raise
            row.update(exit=proc.returncode, stdout=proc.stdout.decode(), stderr=proc.stderr.decode())
            if error is None:
                assert proc.returncode == 0 and not proc.stderr and json.loads(proc.stdout) == want, row
            else:
                assert proc.returncode == 2 and not proc.stdout and error in row["stderr"], row
            row["passed"] = True
            save()
        assert sha(binary) == result["binary_sha256"] and sha(APP) == source_sha
        result["status"] = "completed"
    except BaseException as exc:
        result.update(status="failed", error=repr(exc))
        raise
    finally:
        result.update(total=len(rows), passed=sum(row["passed"] for row in rows))
        save()
    print(json.dumps(dict(status=result["status"], total=result["total"], passed=result["passed"])))


if __name__ == "__main__":
    main()
