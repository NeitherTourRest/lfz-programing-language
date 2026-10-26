from pathlib import Path
import re
for label in ['old','current']:
 asm=next((Path('Temp/runtime-t12/perf-lab')/label/'target/release/deps').glob('*.s')).read_text()
 print(label)
 for fn in ['eval_expr_inner','exec_assign','lookup','assign_name','exec_while','exec_block','exec_stmt_inner','eval_rhs','apply_binary']:
  matches=list(re.finditer(r'^([^\n]*'+fn+r'[^\n]*):\s*$',asm,re.M))
  for m in matches[:1]:
   body=asm[m.end():];body=body[:body.find('.seh_endproc')]
   print(fn, 'lines',len(body.splitlines()),'stack',re.findall(r'subq\s+\$(\d+), %rsp',body)[:1], 'calls',len(re.findall(r'\bcallq',body)))
