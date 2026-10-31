from pathlib import Path
import hashlib,json,zipfile,shutil
R=Path.cwd();W=R/'Temp/T18-B-tooling';E=R/'docs/tooling/evidence/T18-B';E.mkdir(exist_ok=True)
for p in W.iterdir():
 if p.is_file() and p.name!='tls-key.pem':shutil.copy2(p,E/p.name)
(E/'.gitattributes').write_text('* -text\n',encoding='utf-8')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
rows=json.loads((W/'watched-summary.json').read_text(encoding='utf-8'));assert len(rows)==12 and all(r['exit']==0 for r in rows) and sum(r['passed'] for r in rows)==667
sources=sorted((R/'src').rglob('*.rs'))+[R/'Cargo.toml',R/'Cargo.lock',R/'tests/cli_t18b.rs'];exe=R/'target/release/lfz.exe';assert sha(exe)=='18d328d9e48780f8e6ae638f9ad24f53186d94f2207ca24da8928cbbba0d0916'
with zipfile.ZipFile(E/'candidate-snapshot.zip','x',zipfile.ZIP_DEFLATED) as z:
 for p in sources+[exe]:z.write(p,p.relative_to(R).as_posix())
(E/'snapshot.json').write_text(json.dumps({'members':len(sources)+1,'bytes':(E/'candidate-snapshot.zip').stat().st_size,'sha256':sha(E/'candidate-snapshot.zip')},indent=2)+'\n',encoding='utf-8')
d={'date':'2026-10-31','status':'author PASS; independent/public pending','binary':{'path':'target/release/lfz.exe','bytes':exe.stat().st_size,'sha256':sha(exe)},'tests':{'total':667,'failed':0,'ignored':0,'harnesses':rows,'watchdog_seconds_per_harness':90,'lifecycle_final':{'passed':7,'watchdog_seconds':40,'normal_repeats':20,'synchronized_cancel_completion_races':40},'tls':'self-signed rejected WinHTTP12175; server application bytes zero; trusted TLS success not tested'},'source_hashes':{p.relative_to(R).as_posix():sha(p) for p in sources},'evidence':{p.name:sha(p) for p in sorted(E.iterdir()) if p.is_file() and p.name!='manifest.json'}}
(E/'manifest.json').write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
p=R/'docs/reports/T18-B-tooling.md';s=p.read_text(encoding='utf-8-sig').replace('两轮完整harness不并发','两次编译与生命周期测试不并发');p.write_text(s,encoding='utf-8')
(R/'.opencode/team/agents/tooling-dev/STATUS.md').write_text('''# tooling-dev — 工作状态
> 最后更新：2026-10-31 00:15 by tooling-dev

## 当前状态
T18-B HTTP作者实现与验收完成，源码/编译停止，交root公开49/独立验收。仅本轮，不T19/automation，不dist/Git。

## 产出
- http_control.rs纯owned ABI+共用验证helper、私有Windows async单owner/有界completion/稳定buffer与context/安全close最后回调握手；异常展开也先回收。非Windows Unsupported。
- CLI独立allow-http与首类别、CODEargv保护、双diagnostic宿主门禁；获批runner最小双列表fail-closed。旧process文案/旧测试不改。
- 先旧binary红，第一编译生命周期4/4保留；第二最终7/7，含原20normal及新增40Barrier真实cancel完成race，active0/closing断言；close注入仍真实回收、保Deadline。
- 新完整release12harness667/667，0失败忽略，build0。exe1029120B/SHA18D328D9E48780F8E6AE638F9AD24F53186D94F2207CA24DA8928CBBBA0D0916。
- API原始头UTF8/u16le及public-header-witness.json（固定wire头59+body2=N61，实际原文）；TLS真实自签拒绝12175、server应用bytes0，无trust改动/可信TLS成功声明。首openssl config失败保留。
- docs/reports/T18-B-tooling.md、docs/tooling/http-contract.md；docs/tooling/evidence/T18-B/递归manifest/精确ZIP/全部阶段日志。旧A证据保持，临时测试私钥未入ZIP。

## 下一步/限制
root接公开/独立矩阵（方法body/redirect/请求次数/坏协议等），作者7项不冒称全公开覆盖；任何产品修源须重编新hash。Windows不代表Linux支持，OS异常回收不硬实时，R304OPEN。无额外阻塞。
''',encoding='utf-8')
with (R/'.opencode/team/agents/tooling-dev/JOURNAL.md').open('a',encoding='utf-8') as f:f.write('\n### [2026-10-31 00:15] T18-B作者实现收口\nowned HTTP backend/CLI/runner双诊断门禁，先红再首4/最终7生命周期，40真实Barrier取消完成race及20normal，TLS自签拒绝12175/appbytes0，API头原件publicN61。唯一release全12harness667/667/build0，新1029120B/SHA18D328D9E48780F8E6AE638F9AD24F53186D94F2207CA24DA8928CBBBA0D0916。永久递归manifest+ZIP，失败/阶段日志保全；源静默交root公开独立，不dist/Git/automation/T19。\n')
print(json.dumps({'binary':d['binary'],'snapshot':json.loads((E/'snapshot.json').read_text())}))
