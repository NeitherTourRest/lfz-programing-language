from pathlib import Path
import json,hashlib,zipfile,shutil
R=Path.cwd();W=R/'Temp/T18-A-close';E=R/'docs/tooling/evidence/T18-A-close';E.mkdir(exist_ok=True)
for p in W.iterdir():
 if p.is_file():shutil.copy2(p,E/p.name)
(E/'.gitattributes').write_text('* -text\n',encoding='utf-8')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
rows=json.loads((E/'watched-summary.json').read_text(encoding='utf-8'));assert len(rows)==11 and all(r['exit']==0 for r in rows) and sum(r['passed'] for r in rows)==649
sources=sorted((R/'src').rglob('*.rs'))+[R/'Cargo.toml',R/'Cargo.lock',R/'tests/cli_t18a.rs'];exe=R/'target/release/lfz.exe';fixture=R/'Temp/T18-A-tooling/fixture.exe'
assert sha(exe)=='4093714b2dc61bb72dbc4f43db67a798a9bcaf152ad33a11d4d7457d1f7d9c86'
with zipfile.ZipFile(E/'candidate-snapshot.zip','x',zipfile.ZIP_DEFLATED) as z:
 for p in sources+[exe,fixture]:z.write(p,p.relative_to(R).as_posix())
(E/'snapshot.json').write_text(json.dumps({'members':len(sources)+2,'sha256':sha(E/'candidate-snapshot.zip'),'bytes':(E/'candidate-snapshot.zip').stat().st_size},indent=2)+'\n',encoding='utf-8')
d={'date':'2026-10-30','status':'Windows author verification PASS; independent pending','binary':{'path':'target/release/lfz.exe','bytes':exe.stat().st_size,'sha256':sha(exe)},'previous_candidate':'../T18-A/manifest.json; E5F57/646 preserved','tests':{'total':649,'failed':0,'ignored':0,'permitted_harnesses':rows,'outer_watchdog_seconds_per_harness':90,'watchdog_triggered':False},'source_hashes':{p.relative_to(R).as_posix():sha(p) for p in sources},'fixture_sha256':sha(fixture),'evidence':{p.name:sha(p) for p in sorted(E.iterdir()) if p.is_file() and p.name!='manifest.json'}}
(E/'manifest.json').write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
p=R/'docs/reports/T18-A-tooling.md';text=p.read_text(encoding='utf-8').replace('最终候选target/release/lfz.exe984576B','首阶段候选（补关闭诊断前）target/release/lfz.exe984576B')
text+='''
## 关闭诊断补丁与最终作者验收（2026-10-30 18:26）

root要求将CloseHandle本身失败纳入可靠回收。本次run拥有局部Handles注册表，创建成功后立刻登记句柄，必要早关显式close，execute任何Result返回后显式close_all逆序处理剩余句柄，Drop仅意外退出/展开兜底。槽先取空，再单次真实CloseHandle；失败记录独立diagnostics，不重试可能已经消费的句柄值。无生产全局/TLS sink、无Rc跨owner边界。句柄关闭诊断遇正常结果改Io，已有错误原样保留，尤其全局Cancelled仍是不可吞主原因。

新增两unit验证早关/最终逆序关/重复close_all不doubleclose，OS Wait确认注入句柄确已关闭；正常完成、CreateProcess失败、Assign挂起纳管失败、pipe创建失败均保全close诊断。真实fixture marker握手后另一线程取消，验证close诊断不覆盖Cancelled。close注入仅报告故障，仍执行真实CloseHandle；不制造内核损坏或真实句柄泄漏，不声称覆盖所有内核异常。

runtime owner另修非Windows合法绝对路径校验以进入Unsupported并新增1unit，本文不宣称Linux执行验收。重新cargo test --release --no-run与cargo build --release均0；新十一release harness649/649（487+53+28+12+12+7+5+9+6+15+15），0失败/忽略，外watchdog未触发。Windows Unsupported dead_code与环境canonicalize warning仍保留。

最终作者候选target/release/lfz.exe984064B，SHA256 `4093714b2dc61bb72dbc4f43db67a798a9bcaf152ad33a11d4d7457d1f7d9c86`。新证据docs/tooling/evidence/T18-A-close/manifest.json递归绑定源码与日志，candidate-snapshot.zip精确保全。旧T18-A的E5F57候选、646及失败环境日志全部保留。本角色源码/编译已停止，交root独立验收；未动dist/Git，仍仅本轮，不HTTP。
''';p.write_text(text,encoding='utf-8')
status=R/'.opencode/team/agents/tooling-dev/STATUS.md'
status.write_text('''# tooling-dev — 工作状态
> 最后更新：2026-10-30 18:26 by tooling-dev

## 当前状态
T18-A关闭诊断补丁完成，新649/649作者验收PASS，源码/编译已停止，交root独立验收。仅本轮交付后停止，不HTTP/后续，不覆盖dist/Git。

## 产出与证据
- run独占Handles注册表，必要早关显式close，所有Result路径最终显式逆序close_all，Drop仅异常兜底；消耗槽后单次CloseHandle不doubleclose。关闭失败加入owned diagnostics，正常不能假成功，已有Err/Cancelled主因保持。
- 新2unit覆盖真实关闭确认、正常/启动失败/Assign挂起拒绝/pipe失败诊断、真实marker握手活动取消仍保主因；注入仍真实close，无故意泄漏。原生Job/白名单/双流有界/后代回收/权限session/JSON/runner门禁原实现保留。
- runtime修非Windows绝对路径后进入Unsupported，新1unit同编。唯一cargo test --release --no-run/build0，十一harness649/649=487+53+28+12+12+7+5+9+6+15+15，0失败忽略，watchdog未触发。
- 新984064B/hash4093714B2DC61BB72DBC4F43DB67A798A9BCAF152AD33A11D4D7457D1F7D9C86。证据docs/tooling/evidence/T18-A-close/含递归manifest/精确ZIP/日志；docs/reports/T18-A-tooling.md追加收口。E5F57/646阶段ZIP及日志原样保全。

## 待协调
root接独立/公开/指南验收，任何后续源码改动必须新编/hash。Windows结果不代表Linux支持；非Windows仅Unsupported，未实测。Unsupported dead_code/环境警告保留。R304OPEN，不正式tag，无其他阻塞。
''',encoding='utf-8')
j=R/'.opencode/team/agents/tooling-dev/JOURNAL.md'
with j.open('a',encoding='utf-8') as f:f.write('\n### [2026-10-30 18:26] T18-A关闭诊断补丁收口\nrun独占Handles注册表显式早关/逆序最终关，Drop仅兜底，CloseHandle失败诊断及真实关闭注入，2unit覆盖正常/启动/Assign/pipe失败及真实取消主因。runtime平台修复1unit同编，release no-run/build0，十一harness649/649，新984064B/SHA4093714B2DC61BB72DBC4F43DB67A798A9BCAF152AD33A11D4D7457D1F7D9C86。docs/tooling/evidence/T18-A-close/递归manifest+ZIP，旧E5/646全部保留。源码静默交root，仅本轮，不HTTP/Git/dist。\n')
print(json.dumps({'binary':d['binary'],'zip':json.loads((E/'snapshot.json').read_text()),'total':649},ensure_ascii=False))
