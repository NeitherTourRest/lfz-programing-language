from pathlib import Path
import hashlib,json
root=Path.cwd()
evidence=Path(__file__).resolve().parent
sha=lambda data: hashlib.sha256(data).hexdigest()
expected="76ac4cb51fe8ff1a67cd697540e483f50759e4fd67dce00636781481d492f11e"
markers=[f"CASE{i}<{s}>".encode() for i,s in enumerate(["中文雪","😀","𠀀","é","👩‍💻"])]
native=json.loads((evidence/"ordinary-console.json").read_bytes())
assert native["binary_sha256"]==expected and len(native["results"])==6
assert "harness_error" not in native["handles"]
for cp in (437,936,65001):
    for mode in ("wide","run","repl"):
        prefix=evidence/f"{cp}-{mode}"
        run=json.loads(prefix.with_suffix(".result.json").read_bytes())
        handles=json.loads(prefix.with_suffix(".handles.json").read_bytes())
        raw=prefix.with_suffix(".raw.bin").read_bytes()
        raw.decode("utf8")
        assert run["binary_sha256"]==expected and run["exit"]==0
        assert handles["input_cp"]==handles["output_cp"]==cp
        assert all(h["mode_ok"] for h in handles["handles"])
        assert all(raw.count(m)==1 for m in markers),prefix
        if mode=="repl":
            payload=prefix.with_suffix(".input.bin").read_bytes()
            assert payload.isascii() and all(m not in payload for m in markers)
    ordinary=(evidence/f"{cp}-ordinary-out.utf16le").read_bytes()
    direct=(evidence/f"{cp}-wide.utf16le").read_bytes()
    assert ordinary==direct
    assert "EMOJI<�> CJK<�> BMP<中文雪>" in direct.decode("utf-16-le")
    for row in (r for r in native["results"] if r["cp"]==cp):
        assert row["exit"]==0 and row["actual_cp"]==cp and row["raw_sha256"]==sha(direct)
print("45/45 ConPTY markers, 3/3 native byte comparisons")
