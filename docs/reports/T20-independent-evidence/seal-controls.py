import pathlib,json,hashlib,shutil
r=pathlib.Path.cwd(); w=r/'Temp/T20-verifier'; p=r/'docs/validation/agent-benchmark-v3/v3-t20-r01'; e=r/'docs/reports/T20-independent-evidence'; e.mkdir(exist_ok=True)
def sha(f):return hashlib.sha256(f.read_bytes()).hexdigest()
def read(f):return json.loads(f.read_text(encoding='utf-8-sig'))
m=p/'manifest.draft.json'; files=read(m)['files']; assert len(files)==38 and all(sha(p/n)==h for n,h in files.items()); b=sha(r/'target/release/lfz.exe'); assert b=='a545d0c294eb85276abc5556f297434806ea1def598192ee664593550c579a3c'
refs=['default-ref-C01','os-ref-C02','default-ref-C03','os-ref-C04']; negatives=['default-C01-bad-cap','source-C01-collect-preload','os-C02-caller-rate','os-C02-no-home-update','default-C03-success-nil-false','default-C03-budget-fake-success','os-C04-wrong-order','os-C04-treat-404-error']; bindings={}
for n in refs+negatives+['os-ref-C04-close-injected-final']:
 f=w/(n+'.json'); v=read(f); assert v['binary_sha256']==b; assert v['functional_axis']==('PASS' if n in refs else 'FAIL'); assert v['protocol_axis']=='UNKNOWN'; bindings[n]=sha(f); shutil.copyfile(f,e/f.name)
assert read(w/'source-C01-collect-preload.json')['source_axis']=='FAIL'
v=read(w/'os-ref-C04-close-injected-final.json');assert v['server_closed'] is False and all(row['pass'] for row in v['runs']);assert read(w/'os-ref-C04.json')['server_closed'] is True
rows=read(w/'default-ref-C01.json')['runs'];assert [len(x['committed_rows'])for x in rows if x['name'] in ('data10','data11')]==[32,32]
budget=read(w/'budget-controls.json');assert len(budget)==9 and all(x['actual']==x['expected'] for x in budget)
for n in ['budget-controls.json','C01-collect-source-review.json','close-injection.py','close-injection-first.py','os-ref-C04-close-false.json','default-ref-C04.log','default-ref-C02.json','controls-default-summary.json']:
 f=w/n;shutil.copyfile(f,e/n);bindings[n]=sha(f)
receipt=dict(reviewer='verifier_t20',binary_sha256=b,binary_bytes=(r/'target/release/lfz.exe').stat().st_size,draft_sha256=sha(m),draft_files_verified=len(files),draft_files=files,positive_functional='4/4',negative_expected='8/8',budget_controls='9/9',server_close_false='FAIL with all product rows PASS',source_preload='FAIL confirmed by source review',student_protocol='UNKNOWN until real trace is reconciled',evidence_sha256=bindings)
(e/'v3-controls-receipt.json').write_text(json.dumps(receipt,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(receipt['draft_sha256'],sha(e/'v3-controls-receipt.json'))
