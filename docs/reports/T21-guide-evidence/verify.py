"""Verify the T21 human guide against the final candidate."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
GUIDES = [ROOT / "docs/guide/README.md", ROOT / "docs/guide/formatting.md"]
FIXTURE = ('\ufeff#42\r\n\tprint("你好")  \r\n'
           '\t// comment  \t\r\n\twriteFile("ran.txt", "ran") \t').encode("utf-8")
EXPECTED = ('\ufeff#42\r\n print("你好")\r\n'
            ' // comment  \t\r\n writeFile("ran.txt", "ran")').encode("utf-8")
SAVE_COMMAND = 'lfz.exe fmt "source file.lfz" > "formatted file.lfz"'


def sha(data):
    return hashlib.sha256(data).hexdigest()


def guide_hashes():
    return {p.relative_to(ROOT).as_posix(): sha(p.read_bytes()) for p in GUIDES}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--sha256")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    guides = guide_hashes()
    if args.prepare_only:
        assert SAVE_COMMAND in GUIDES[1].read_text(encoding="utf-8")
        print(json.dumps({"prepared": 2, "executed": False, "docs": guides}))
        return
    if not (args.binary and args.sha256 and args.output):
        parser.error("--binary, --sha256 and --output are required")
    binary = args.binary.resolve(strict=True)
    output = args.output.resolve()
    assert binary.is_relative_to(ROOT) and binary.name.lower() == "lfz.exe"
    assert output.is_relative_to(ROOT) and not output.exists()
    product_hash = sha(binary.read_bytes())
    assert product_hash == args.sha256.lower()
    work = ROOT / "Temp/T21-guide" / output.stem
    work.mkdir(parents=True)
    original = work / "source file.lfz"
    formatted = work / "formatted file.lfz"
    original.write_bytes(FIXTURE)
    env = os.environ.copy()
    env["PATH"] = str(binary.parent) + os.pathsep + env["PATH"]
    rows = []

    def record(name, process, result):
        rows.append({
            "name": name,
            "exit": process.returncode,
            "stdout_hex": process.stdout.hex(),
            "stderr_hex": process.stderr.hex(),
            "output_hex": result.hex(),
            "pass": process.returncode == 0 and not process.stderr
                    and result == EXPECTED and original.read_bytes() == FIXTURE
                    and not (work / "ran.txt").exists(),
        })

    result = subprocess.run([binary, "fmt", original.name], cwd=work,
                            capture_output=True, timeout=10)
    record("stdout", result, result.stdout)
    result = subprocess.run("cmd.exe /d /c " + SAVE_COMMAND, cwd=work,
                            env=env, capture_output=True, timeout=10)
    record("cmd-redirect", result, formatted.read_bytes() if formatted.exists() else b"")
    evidence = {
        "binary_sha256": product_hash, "binary_bytes": binary.stat().st_size,
        "final_binary_sha256": sha(binary.read_bytes()), "docs": guides,
        "final_docs": guide_hashes(), "executed": True, "runs": rows,
    }
    evidence["passed"] = sum(row["pass"] for row in rows)
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("x", encoding="utf-8") as log:
        json.dump(evidence, log, ensure_ascii=False, indent=2)
        log.write("\n")
    print(json.dumps({"passed": evidence["passed"], "total": len(rows)}))
    assert evidence["final_binary_sha256"] == product_hash and evidence["final_docs"] == guides
    raise SystemExit(evidence["passed"] != len(rows))


if __name__ == "__main__":
    main()
