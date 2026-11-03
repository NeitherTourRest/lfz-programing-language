from pathlib import Path
import hashlib,json,zipfile,shutil
R=Path.cwd();W=R/'Temp/T19-tooling';E=R/'docs/tooling/evidence/T19';E.mkdir(exist_ok=True)
for p in W.iterdir():
 if p.is_file():shutil.copy2(p,E/p.name)
(E/'.gitattributes').write_text('* -text\n',encoding='utf-8')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
rows=json.loads((W/'watched-summary.json').read_text(encoding='utf-8'));assert len(rows)==12 and all(r['exit']==0 for r in rows) and sum(r['passed'] for r in rows)==684
assert json.loads((W/'green-valid-input.json').read_text(encoding='utf-8-sig'))['actualExit']==0
sources=sorted((R/'src').rglob('*.rs'))+sorted((R/'tests').rglob('*.rs'))+[R/'Cargo.toml',R/'Cargo.lock'];exe=R/'target/release/lfz.exe';assert sha(exe)=='5d2219f88db8b3196cac259941fc0a8a3b489ef03519bc7e20415f8edf6388e5'
with zipfile.ZipFile(E/'candidate-snapshot.zip','x',zipfile.ZIP_DEFLATED) as z:
 for p in sources+[exe]:z.write(p,p.relative_to(R).as_posix())
(E/'snapshot.json').write_text(json.dumps({'members':len(sources)+1,'bytes':(E/'candidate-snapshot.zip').stat().st_size,'sha256':sha(E/'candidate-snapshot.zip')},indent=2)+'\n',encoding='utf-8')
d={'date':'2026-11-04','status':'author full regression PASS; public/independent/scale pending','binary':{'path':'target/release/lfz.exe','bytes':exe.stat().st_size,'sha256':sha(exe)},'tests':{'total':684,'failed':0,'ignored':0,'harnesses':rows,'watchdog_seconds_per_harness':90,'test_threads_per_harness':1,'first_reader':{'passed':7,'log':'reader-first.txt'},'retained_failure':'first-final-failed-lib.txt: 515 passed/1 failed, author new test LFZ TwoStatements; fixed newline only in new test'},'source_hashes':{p.relative_to(R).as_posix():sha(p) for p in sources},'evidence':{p.name:sha(p) for p in sorted(E.iterdir()) if p.is_file() and p.name!='manifest.json'}}
(E/'manifest.json').write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
p=R/'docs/reports/T19-tooling.md';s=p.read_text(encoding='utf-8-sig');s+='''
## 最终作者回执（2026-11-04 01:23）

修后release no-run/build均exit0，新12harness684/684=516+56+28+12+12+7+5+9+6+3+15+15，0失败/忽略；旧A关闭probe仅正常OS许可同harness实跑，未再次默认sandbox诱发abort。每程序90秒外watchdog未触发。reader十一unit、runtime五unit和core新增消息测试均实际同编执行，故障/spy与真实存活host重开路径均通过。首515/1原始失败和修正依据保全。

最终1041920B/SHA256 `5d2219f88db8b3196cac259941fc0a8a3b489ef03519bc7e20415f8edf6388e5`。有效输入红(old dist NameError2)/绿(new exe0)同source见证均落盘；初无效路径红仅作名字缺失阶段记录。永久docs/tooling/evidence/T19/manifest.json递归绑定全部src及Rust tests、Cargo、最终exe和日志；candidate-snapshot.zip精确保全。警告Unsupported与Io未读取字段、环境canonicalize保留，未宣称零warning。

源码/编译停止交root；公开/独立/规模oracle与perf尚待其验收，不在本报告冒称完成。未覆盖dist或Git，原用户块保留。
''';p.write_text(s,encoding='utf-8')
(R/'.opencode/team/agents/tooling-dev/STATUS.md').write_text('''# tooling-dev — 工作状态
> 最后更新：2026-11-04 01:23 by tooling-dev

## 当前状态
T19 reader/lib作者实现及最终684/684回归完成，源码/编译已停止，交root公开/独立与perf。仅T19，不T20/automation；未dist/Git。

## 产出/证据
- src/line_reader.rs精确owned ABI，8KiB固定缓冲/追加前raw cap+capacity限制、LF/CRLF/EOF/BOM/NUL/跨块UTF8/overflow、普通file双metadata、nativecancel/deadline-only，finish/RAII，无生产FFI/close diag/sync_all。
- reader11unit+runtime5unit同编实际通过：活callback share0错误32负控，宿主/traced仍存活所有Outcome后排他成功；故障注入metadata/open/postopen/read、Drop spy恰一次，暖机50+1000次handle plateau；非Windows资源实测未声明。
- 首reader7/7保留；首次最终lib515/1因本人新test while/true同行TwoStatements，保完整失败，仅修新增test换行，未改parser/runtime/旧tests。随后全12harness684/684，0失败忽略，正常OS许可串行；watchdog未触发，build0。
- 最终1041920B/SHA5D2219F88DB8B3196CAC259941FC0A8A3B489EF03519BC7E20415F8EDF6388E5。旧dist有效输入红→同source新产物绿0真实记录。docs/tooling/evidence/T19/递归manifest+精确ZIP，docs/reports/T19-tooling.md齐。

## 下一步/限制
root接公开/独立，perf统一最终hash/no并发窗口，tooling不重复10/100/300MiB大fixture。std Drop不报告OS close失败，磁盘阻塞不硬截止；R304OPEN，无性能通过声明。任何源改须新编/hash。无其它阻塞。
''',encoding='utf-8')
with (R/'.opencode/team/agents/tooling-dev/JOURNAL.md').open('a',encoding='utf-8') as f:f.write('\n### [2026-11-04 01:23] T19实现最终684通过收口\nreader/lib实现，reader11+runtime5+core新测试同编全验；首515/1新增测试TwoStatements失败完整保全，仅改新增test换行。最终12harness684/684/build0，新1041920B/SHA5D2219F88DB8B3196CAC259941FC0A8A3B489EF03519BC7E20415F8EDF6388E5。真实同host活句柄负控与全部Outcome返回后排他/Drop spy/故障/plateau通过，有效输入旧红新绿。递归源码Rust测试manifest与精确ZIP落docs/tooling/evidence/T19，源/编译静默交root公开独立/perf，不dist/Git/T20/automation。\n')
print(json.dumps({'binary':d['binary'],'snapshot':json.loads((E/'snapshot.json').read_text())}))
