from pathlib import Path
import hashlib,json,zipfile,shutil,datetime
R=Path.cwd();W=R/'Temp/T21-tooling';E=R/'docs/tooling/evidence/T21';E.mkdir(exist_ok=True)
for p in W.iterdir():
 if p.is_file():shutil.copy2(p,E/p.name)
(E/'.gitattributes').write_text('* -text\n',encoding='utf-8')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
rows=json.loads((W/'watched-summary.json').read_text(encoding='utf-8'));assert len(rows)==14 and sum(r['passed'] for r in rows)==715 and all(r['exit']==0 for r in rows)
exe=R/'target/release/lfz.exe';expected='76ac4cb51fe8ff1a67cd697540e483f50759e4fd67dce00636781481d492f11e';assert sha(exe)==expected
assert json.loads((W/'position-trace-witness.json').read_text(encoding='utf-8'))['candidate_sha256']==expected
sources=sorted((R/'src').rglob('*.rs'))+sorted((R/'tests').rglob('*.rs'))+[R/'Cargo.toml',R/'Cargo.lock']
with zipfile.ZipFile(E/'candidate-snapshot.zip','x',zipfile.ZIP_DEFLATED) as z:
 for p in sources+[exe]:z.write(p,p.relative_to(R).as_posix())
snapshot={'members':len(sources)+1,'bytes':(E/'candidate-snapshot.zip').stat().st_size,'sha256':sha(E/'candidate-snapshot.zip')};(E/'snapshot.json').write_text(json.dumps(snapshot,indent=2)+'\n',encoding='utf-8')
manifest={'date':'2026-11-13','status':'author PASS; independent pending','binary':{'path':'target/release/lfz.exe','bytes':exe.stat().st_size,'sha256':expected},'tests':{'total':715,'failed':0,'ignored':0,'harnesses':rows,'watchdog_seconds_per_harness':90,'threads':1},'source_hashes':{p.relative_to(R).as_posix():sha(p) for p in sources},'evidence':{p.name:sha(p) for p in sorted(E.iterdir()) if p.is_file() and p.name!='manifest.json'}}
(E/'manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
stamp=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).strftime('%Y-%m-%d %H:%M')
status=R/'.opencode/team/agents/tooling-dev/STATUS.md'
status.write_text(f'''# tooling-dev — 工作状态
> {stamp}

T21 作者实现和验收完成。`src/formatter.rs`、`src/lib.rs`、`src/cli.rs` 与 `tests/cli_t21.rs` 已落。core 停源后统一 release 构建，14 组 715/715 通过。候选 1117184B，SHA256 `{expected}`。

保真 oracle 核对原字节、幂等、合法/错误输入、模块独立格式化和旧裸 fmt CLI。同路径原脚本与格式化脚本的可观察行列、traceback、NameError JSON 和文件效果一致。报告 `docs/reports/T21-tooling.md`，递归清单和快照位于 `docs/tooling/evidence/T21/`。

源码与构建已冻结，交 root 独立验收。R304 OPEN。
''',encoding='utf-8')
with (status.parent/'JOURNAL.md').open('a',encoding='utf-8') as f:f.write(f'\n### [{stamp}] T21 格式化作者验收\n格式化 API、原始参数分派与新测试已完成。旧候选红测保留，定向 8 项及完整 715 项通过。候选 1117184B，SHA256 `{expected}`。真实脚本前后错误位置、traceback 与文件效果一致。递归清单与候选 ZIP 保全，源冻结交 root 独立验收。\n')
print(json.dumps(snapshot))
