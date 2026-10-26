"""T12 public-process acceptance. Standard library; inputs/evidence stay in repository."""
import argparse,hashlib,json,os,subprocess,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def lit(s):
    parts=[]; chunk=''
    for c in s:
        if ord(c)<32 and c not in '\n\r\t':
            if chunk:parts.append('"'+chunk+'"');chunk=''
            parts.append('chr('+str(ord(c))+')')
        else:chunk+={'\\':'\\\\','"':'\\"','$':'\\$','\n':'\\n','\r':'\\r','\t':'\\t'}.get(c,c)
    if chunk or not parts:parts.append('"'+chunk+'"')
    return '('+' + '.join(parts)+')'
def main():
    ap=argparse.ArgumentParser();ap.add_argument('--lfz',default='target/debug/lfz.exe');ap.add_argument('--output',required=True);ap.add_argument('--full-suite',action='store_true');o=ap.parse_args()
    exe=(ROOT/o.lfz).resolve();out=(ROOT/o.output).resolve()
    for p in [exe,out]:
        if not p.is_relative_to(ROOT):ap.error('paths must remain inside repository')
    if out.exists():ap.error('output must be a new path')
    work=ROOT/'Temp/T12/test-engineer';work.mkdir(parents=True,exist_ok=True)
    env=os.environ.copy();env['TEMP']=env['TMP']=str(work)
    rows=[]
    def run(name,args,stdin=b'',code=0,stdout=None,error=None,message=None,file=None,line=None,col=None):
        t=time.perf_counter();r=subprocess.run([str(exe),*args],input=stdin,stdout=subprocess.PIPE,stderr=subprocess.PIPE,cwd=ROOT,env=env,timeout=10)
        text=r.stdout.decode('utf-8');err=r.stderr.decode('utf-8');checks={'exit':r.returncode==code}
        obj=None
        if error is not None or file is not None or message is not None:
            try:obj=json.loads(text);checks['single_json']=len(text.splitlines())==1 and isinstance(obj,dict)
            except (ValueError,TypeError):checks['single_json']=False
            if obj is not None:
                if error is not None:checks['error']=obj.get('error')==error
                if message is not None:checks['message']=obj.get('message')==message
                if file is not None:checks['file']=obj.get('file')==file
                if line is not None:checks['line']=obj.get('line')==line
                if col is not None:checks['col']=obj.get('col')==col
        if stdout is not None:checks['stdout']=text==stdout
        row={'name':name,'args':args,'stdin_hex':stdin.hex(),'expected':{'exit':code,'stdout':stdout,'error':error,'message':message,'file':file,'line':line,'col':col},'exit':r.returncode,'stdout':text,'stderr':err,'checks':checks,'passed':all(checks.values()),'elapsed_seconds':time.perf_counter()-t}
        rows.append(row);return row,obj
    payload=['','中文🙂','--json','-e','--','--help','test','a b','${x}']
    # Avoid requiring first-class JSON builtin: compare deterministic serialized argv directly.
    body='assert(jsonStringify(argv()) == '+lit(json.dumps(payload,ensure_ascii=False,separators=(',',':')))+', "args")\nlet old=argv()\nold[0]="changed"\nassert(argv()[0]=="", "fresh")\nlet f=argv\nassert(f()==argv(),"alias")\nlet fs=[f]\nassert(fs[0]()==argv(),"container")\nlet h={"f":f}\nassert(h["f"]()==argv(),"struct_call")\nassert(map(fn(g){g()},[f,f])==[argv(),argv()],"hof")\nprint("ok")'
    for prefix in [[],['run']]:
        run('argv_inline_'+str(len(prefix)),prefix+['-e',body,'--',*payload],stdout='ok\n')
        run('argv_stdin_'+str(len(prefix)),prefix+['-','--',*payload],body.encode(),stdout='ok\n')
    fixture=work/'args.lfz';fixture.write_text('#42\n'+body,encoding='utf-8')
    for prefix in [[],['run']]:run('argv_file_'+str(len(prefix)),prefix+[str(fixture),'--',*payload],stdout='ok\n')
    for name,args in [('empty_inline',['-e','']),('empty_run_inline',['run','-e','']),('empty_stdin',['-']),('empty_stdin_run',['run','-']),('trailing_separator',['-e','assert(argv()==[])','--'])]:run(name,args,stdout='')
    for name,args in [('missing_inline',['-e']),('missing_run_inline',['run','-e']),('extra_inline',['-e','1','extra']),('extra_file',['run',str(fixture),'extra']),('separator_before_entry',['--','x']),('separator_without_run_entry',['run','--','x']),('no_args',[])]:run(name,args,code=2,stdout='')
    for source in ['--json','--','--help']:
        run('code_token_literal_'+source,['--json','-e',source],code=2,file='<command>')
    for args in [['--json','-e','print("payload")'],['-e','print("payload")','--json'],['--json','run','-e','print("payload")','--json'],['-e','print("--json")','--','--json']]:
        expected='--json\n' if '--' in args else '{"ok":true}\n'
        row,_=run('json_boundary_'+str(len(rows)),args,stdout=expected)
        if expected.startswith('{'):row['checks']['payload_stderr']='payload\n'==row['stderr'];row['passed']=all(row['checks'].values())
    row,obj=run('stdin_bad_utf8',['--json','-'],b'\xff',code=2,error='IOError',message='无法读取标准输入',file='<stdin>')
    if obj is not None:row['checks']['no_location']=obj.get('line') is None and obj.get('col') is None and obj.get('traceback')==[];row['passed']=all(row['checks'].values())
    run('stdin_runtime_location',['--json','-'],b'let x=1\n1/0\n',code=2,error='ZeroDivisionError',file='<stdin>',line=2)
    run('inline_runtime_location',['--json','-e','1/0'],code=2,error='ZeroDivisionError',file='<command>',line=1)
    run('stdin_input_eof',['--json','-'],b'input()\n',code=2,error='IOError',file='<stdin>')
    run('inline_input_data',['-e','print(input())'],b'hello\n',stdout='hello\n')
    run('inline_not_auto_print',['-e','40+2'],stdout='')
    run('stdin_bom_crlf',['-'],b'\xef\xbb\xbfprint(42)\r\n',stdout='42\n')
    parse_cases=[('',0,'不是合法的 JSON'),('01',1,'不是合法的 JSON'),('+1',0,'不是合法的 JSON'),('1.',2,'不是合法的 JSON'),('1e+',3,'不是合法的 JSON'),('[1,]',3,'不是合法的 JSON'),('{"a":1,}',7,'不是合法的 JSON'),('true false',5,'不是合法的 JSON'),('"中"x',5,'不是合法的 JSON'),('{"a":1,"a":2}',7,'对象键重复'),('{"a":1,"\\u0061":2}',7,'对象键重复'),('{"中":0,"中":1}',9,'对象键重复'),('9223372036854775808',0,'整数超出 i64 范围'),('-9223372036854775809',0,'整数超出 i64 范围'),('1e9999',0,'浮点数超出有限范围'),('['*129+'0'+']'*129,128,'嵌套深度超过 128 层'),('\ufeffnull',0,'不是合法的 JSON'),('"\x00"',1,'不是合法的 JSON'),('"\\x41"',2,'不是合法的 JSON'),('"\\ud800"',7,'不是合法的 JSON'),('"\\udc00"',3,'不是合法的 JSON'),('NaN',0,'不是合法的 JSON'),('Infinity',0,'不是合法的 JSON'),('/*x*/1',0,'不是合法的 JSON')]
    parse_cases += [('-',1,'不是合法的 JSON'),('-01',2,'不是合法的 JSON'),('1e',2,'不是合法的 JSON'),('0x1',1,'不是合法的 JSON'),('1.e2',2,'不是合法的 JSON'),('1e++2',3,'不是合法的 JSON'),('null'+chr(11),4,'不是合法的 JSON'),(chr(160)+'null',0,'不是合法的 JSON'),('"'+chr(92)+'ud800'+chr(92)+'u0041"',9,'不是合法的 JSON')]
    for i,(text,offset,reason) in enumerate(parse_cases):
        payload_path=work/'json-input.txt';payload_path.write_text(text,encoding='utf-8',newline='')
        row,_=run('parse_exact_'+str(i),['--json','-e','jsonParse(readFile(argv()[0]))','--',str(payload_path)],code=2,error='ValueError',message=f'JSON 解析失败（字节偏移 {offset}）：{reason}',file='<command>',line=1,col=1)
        row['json_input_utf8_hex']=text.encode('utf-8').hex()
    for i,(body,reason) in enumerate([('jsonStringify(fn(){1})','不支持函数或结构体定义'),('jsonStringify([fn(){1}])','不支持函数或结构体定义'),('jsonStringify(exp(10000))','浮点数必须有限'),('jsonStringify(log(-1))','浮点数必须有限'),('let x={"n":nil}\nx.n=x\njsonStringify(x)','循环引用'),('var x=0\nfor i in range(129){x=[x]}\njsonStringify(x)','嵌套深度超过 128 层')]):
        run('encode_exact_'+str(i),['--json','-e',body],code=2,error='ValueError',message='JSON 编码失败：'+reason)
    values=[None,True,False,0,-1,9223372036854775807,-9223372036854775808,9007199254740993,'中文🙂','${x}','\0\b\f\n\r\t/\\"',[],{},[1,True,None],{'z':1,'a':[False,None],'🙂':{'中':'x'}},{'':0,'a b':1,'${x}':2}]
    for i,value in enumerate(values):
        source=json.dumps(value,ensure_ascii=True,separators=(',',':'))
        expected=json.dumps(value,ensure_ascii=False,sort_keys=True,separators=(',',':'))+'\n'
        run('json_independent_roundtrip_'+str(i),['-e','print(jsonStringify(jsonParse(argv()[0])))','--',source],stdout=expected)
    controls=''.join(chr(i) for i in range(32))
    control_path=work/'controls.bin';control_path.write_bytes(controls.encode('utf-8'))
    run('all_control_escapes_real',['-e','print(jsonStringify(readFile(argv()[0])))','--',str(control_path)],stdout=json.dumps(controls,ensure_ascii=False)+'\n')
    run('float_underflow_negative_zero',['-e','print(jsonStringify(jsonParse("-1e-9999")))'],stdout='-0.0\n')
    run('argv_alias_error',['--json','-e','let f=argv\nf(1)'],code=2,error='TypeError',message='函数 argv 期待 0 个参数，得到 1',file='<command>',line=2)
    if o.full_suite:
        row,obj=run('official_full_suite',['test','--json'])
        try:
            obj=json.loads(row['stdout']);row['checks']['all_suite_pass']=obj['ok'] is True and obj['passed']==obj['total'] and obj['total']>=166 and obj['failed']==obj['errored']==0
        except (ValueError,KeyError,TypeError):row['checks']['all_suite_pass']=False
        row['passed']=all(row['checks'].values())
    report={'schema':'LFZ-T12-BLACKBOX-1','date':'2026-10-25','binary':str(exe.relative_to(ROOT)),'binary_sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'total':len(rows),'passed':sum(r['passed'] for r in rows),'failed':sum(not r['passed'] for r in rows),'cases':rows}
    out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:v for k,v in report.items() if k!='cases'},ensure_ascii=False))
    for row in rows:
        if not row['passed']:print(row['name'],json.dumps(row,ensure_ascii=False))
    return 0 if report['failed']==0 else 1
if __name__=='__main__':raise SystemExit(main())
