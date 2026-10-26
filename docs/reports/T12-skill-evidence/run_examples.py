from pathlib import Path
import subprocess,json,hashlib,re,os,zipfile
ROOT=Path.cwd(); TMP=ROOT/'Temp/ai-dx-t12'; E=ROOT/'docs/reports/T12-skill-evidence'; E.mkdir(exist_ok=True)
TMP.mkdir(parents=True,exist_ok=True)
EXE=ROOT/'dist/lfz.exe'; results=[]
env=dict(os.environ,TEMP=str(TMP),TMP=str(TMP))
def run(name,args,expected_code=0,expected_out=None,inp=None,message=None,error=None,stderr=None):
    p=subprocess.run([str(EXE),*args],input=inp,capture_output=True,timeout=15,env=env,cwd=ROOT)
    out=p.stdout.decode('utf-8'); err=p.stderr.decode('utf-8')
    assert p.returncode==expected_code,(name,p.returncode,out,err)
    if expected_out is not None: assert out==expected_out,(name,out,expected_out)
    if stderr is not None: assert err==stderr,(name,err,stderr)
    if message is not None:
        d=json.loads(out); assert d['message']==message,(name,d)
        assert d['error']==error,(name,d)
    rec=dict(name=name,args=args,stdin_hex=None if inp is None else inp.hex(),exit=p.returncode,stdout=out,stderr=err)
    results.append(rec)
    return rec
s=(ROOT/'.opencode/skills/lfz-programming/SKILL.md').read_text(encoding='utf-8')
section=s[s.index('## 6.2 '):s.index('## 7.')]
code=re.search(r'```lfz\n(.*?)\n```',section,re.S).group(1)+'\n'; (TMP/'job.lfz').write_text(code,encoding='utf-8'); (E/'job.lfz').write_text(code,encoding='utf-8')
payload=json.dumps(dict(title='中文🙂',values=[1,2,3,4]),ensure_ascii=False,separators=(',',':'))
run('documented-config-pipeline',['run',str(TMP/'job.lfz'),'--',payload],expected_out='{"count":2,"title":"中文🙂","total":60}\n',stderr='')
run('documented-argv-literals',['-e','print(jsonStringify(argv()))','--','中文','--json',''],expected_out='["中文","--json",""]\n',stderr='')
run('documented-machine-channel',['-e','print(jsonStringify(argv()))','--json','--','--json'],expected_out='{"ok":true}\n',stderr='["--json"]\n')
run('stdin-code',['run','-','--','输入🙂'],inp=b'print(jsonStringify(argv()))\n',expected_out='["输入🙂"]\n',stderr='')
run('empty-code',['-e',''],expected_out='',stderr='')
run('code-token-json-not-option',['-e','--json'],expected_code=2)
run('stdin-input-eof',['--json','-'],inp=b'input()\n',expected_code=2,message='输入结束（EOF）',error='IOError')
run('stdin-invalid-utf8',['--json','-'],inp=b'\xff',expected_code=2,message='无法读取标准输入',error='IOError')
run('argv-identity-fresh-alias-shadow',['-e','let saved = argv\nlet a = argv()\na[0] = "changed"\nfn shadow(argv) { argv }\nassert(saved == argv)\nassert(type(argv) == "function")\nassert(str(argv) == "<fn argv>")\nassert(shadow(3) == 3)\nprint(jsonStringify(saved()))','--','original'],expected_out='["original"]\n',stderr='')
run('json-callback-lambda',['-e','let xs = ["true", "null", "1.0"] |> map((s) => jsonParse(s))\nprint(jsonStringify(xs))'],expected_out='[true,null,1.0]\n',stderr='')
run('json-int-float-unicode-dollar',['-e','print(jsonStringify(jsonParse(argv()[0])))','--','{"z":9007199254740993,"a":[-0.0,1.0,"中文🙂${not_lfz}","\\u0000"]}'],expected_out='{"a":[-0.0,1.0,"中文🙂${not_lfz}","\\u0000"],"z":9007199254740993}\n',stderr='')
run('json-method-filter',['-e','let s = { keep: 1, method: fn() { 2 } }\nprint(jsonStringify(s))'],expected_out='{"keep":1}\n',stderr='')
run('json-shared-noncycle',['-e','let x = { a: 1 }\nprint(jsonStringify([x,x]))'],expected_out='[{"a":1},{"a":1}]\n',stderr='')
for name,text,reason,offset in [
 ('json-syntax','{"中":1,}','不是合法的 JSON',len('{"中":1,'.encode())),
 ('json-duplicate','{"a":1,"\\u0061":2}','对象键重复',7),
 ('json-int-overflow','9223372036854775808','整数超出 i64 范围',0),
 ('json-float-overflow','1e309','浮点数超出有限范围',0),
 ('json-bom','\ufeffnull','不是合法的 JSON',0),
 ('json-lone-surrogate','"\\ud800"','不是合法的 JSON',7),
 ('json-depth129','['*129+'0'+']'*129,'嵌套深度超过 128 层',128),
]:
 run(name,['--json','-e','jsonParse(argv()[0])','--',text],expected_code=2,message=f'JSON 解析失败（字节偏移 {offset}）：{reason}',error='ValueError')
run('json-depth128',['-e','print(jsonStringify(jsonParse(argv()[0])))','--','['*128+'0'+']'*128],expected_out='['*128+'0'+']'*128+'\n',stderr='')
for name,code,reason in [
 ('json-cycle','let s = {}\ns.me = s\njsonStringify(s)','循环引用'),
 ('json-function-array','jsonStringify([fn() { 1 }])','不支持函数或结构体定义'),
 ('json-nonfinite','jsonStringify(exp(1000))','浮点数必须有限'),
]:
 run(name,['--json','-e',code],expected_code=2,message=f'JSON 编码失败：{reason}',error='ValueError')
# 原有分级示例与最小任务基线，代码来自当前指南，不改原样例。
section=s[s.index('## 7.'):s.index('## 8.')]
for i,block in enumerate(re.findall(r'```lfz\n(.*?)\n```',section,re.S),1):
 if not block.startswith('#42'): continue
 p=TMP/f'old-level-{i}.lfz';p.write_text(block+'\n',encoding='utf-8')
 run(f'old-level-{i}',['run',str(p)])
base=(ROOT/'.opencode/skills/lfz-programming/prompt-template.md').read_text(encoding='utf-8')
block=re.search(r'```lfz\n(#42.*?)\n```',base,re.S).group(1);p=TMP/'sum-even.lfz';p.write_text(block+'\n',encoding='utf-8')
run('minimum-baseline',['run',str(p)],expected_out='6\n',stderr='')
raw=(ROOT/'docs/reports/agent-benchmark-v2/v2-20261024-r01/raw-inputs-submissions.zip').read_bytes()
assert hashlib.sha256(raw).hexdigest()=='90a6a3e3d894e7a813b06463506d5cc540e0a4570d7b005a3fd27ccc6a776d84'
report=dict(date='2026-10-25',scope='作者示例回归；非新Agent盲测/独立产品验收',exe_sha256=hashlib.sha256(EXE.read_bytes()).hexdigest(),exe_bytes=EXE.stat().st_size,passed=len(results),cases=results,old_raw_zip_sha256=hashlib.sha256(raw).hexdigest(),skill_sha256=hashlib.sha256((ROOT/'.opencode/skills/lfz-programming/SKILL.md').read_bytes()).hexdigest())
(E/'results.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(f'{len(results)} passed; binary {report["exe_sha256"]}')
