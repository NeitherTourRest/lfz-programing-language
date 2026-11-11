import sys, pathlib, subprocess, json, hashlib, shutil, importlib.util
r=pathlib.Path.cwd(); p=r/'docs/validation/agent-benchmark-v3/v3-t20-r01'; e=r/'docs/reports/T20-agent-v3-evidence'; w=r/'Temp/agent-benchmark-v3/v3-t20-r01'; H='793caa2c2a006ab6a843b58513040f52ef164ac0ce039f760bd06baa798763b9'; sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest(); read=lambda f:json.loads(f.read_text(encoding='utf-8-sig'))
m=p/'manifest.frozen.json'; assert sha(m)==H; manifest=read(m); assert all(sha(p/n)==h for n,h in manifest['files'].items()); assert all(sha(r/n)==h for n,h in manifest['final_ai_docs'].items()); assert sha(r/manifest['final_binary']['path'])==manifest['final_binary']['sha256']
trace_spec=importlib.util.spec_from_file_location('trace_export',p/'trace_export.py'); te=importlib.util.module_from_spec(trace_spec);trace_spec.loader.exec_module(te)
for task in sys.argv[1:]:
 d=e/task; d.mkdir(parents=True,exist_ok=False); s=w/'students'/task; sub=d/'submission'; sub.mkdir(); dispatch=w/'dispatch-receipts'/(task+'.json'); rec=read(dispatch); assert rec['manifest_sha256']==H; assert all(sha(s/n)==h for n,h in rec['inputs'].items()); shutil.copyfile(dispatch,d/'dispatch.json')
 for f in list(s.rglob('*.lfz'))+[s/'commands.json']:
  if 'selftest-audit' in f.relative_to(s).parts:continue
  target=sub/f.relative_to(s);target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(f,target)
 shutil.copytree(s/'selftest-audit',d/'selftest-audit'); meta=json.loads(subprocess.check_output(['D:/app/ANACONDA/python.exe','-B','Temp/T20-lead/locate-student-session.py',task],cwd=r,encoding='utf8'))
 trace=te.export(pathlib.Path(meta['path']),meta['session'],meta['parent']); (w/(task+'-trace-review-temp.json')).write_text(json.dumps(trace,ensure_ascii=False,indent=2),encoding='utf8')
 src={str(f.relative_to(sub)):sha(f)for f in sub.rglob('*.lfz')}; (d/'submission-hashes.json').write_text(json.dumps(src,indent=2),encoding='utf8')
 print(task,len(src),trace['tool_calls_count'],trace['tool_results_count'],trace['matched_call_ids'],trace['model_contexts'])
