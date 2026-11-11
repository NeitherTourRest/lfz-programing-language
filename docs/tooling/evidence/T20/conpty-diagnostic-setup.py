from pathlib import Path
p=Path('Temp/T20-tooling/conpty.py');s=p.read_text(encoding='utf-8-sig')
s=s.replace('finally:\n if pi.process and wait', '''except Exception as error:
 result['exception']=repr(error)
 if pi.process:
  code=W.DWORD();exitcode(pi.process,C.byref(code));result['exit_at_failure']=code.value
  if code.value==259:
   send('1\\r:quit\\r');result['diagnostic_wait']=wait(pi.process,2000);exitcode(pi.process,C.byref(code));result['exit_after_diagnostic_input']=code.value
 raise
finally:
 if pi.process and wait''')
s=s.replace("'conpty-watchdog.txt'","'conpty-diagnostic-watchdog.txt'").replace("'conpty-raw.bin'","'conpty-diagnostic-raw.bin'").replace("'conpty-text.txt'","'conpty-diagnostic-text.txt'").replace("'conpty-result.json'","'conpty-diagnostic-result.json'")
Path('Temp/T20-tooling/conpty-diagnostic.py').write_text(s,encoding='utf-8')
