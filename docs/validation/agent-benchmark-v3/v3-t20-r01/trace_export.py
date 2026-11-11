"""读取显式指定关联session，永不扫描其它聊天或导出推理/系统消息。"""
import argparse,json,hashlib
from pathlib import Path
def export(path,session,parent,probe=False):
 raw=path.read_bytes();rows=[json.loads(x)for x in raw.decode('utf8').splitlines()if x.strip()]
 meta=next(r['payload']for r in rows if r['type']=='session_meta');assert meta['id']==session
 spawn=meta.get('source',{});spawn=spawn.get('subagent',{}).get('thread_spawn',{})if isinstance(spawn,dict)else{}
 assert spawn.get('parent_thread_id')==parent,'only approved parent/child session'
 contexts=[];calls=[];results=[]
 for r in rows:
  p=r.get('payload',{});t=p.get('type')
  if r['type']=='turn_context':contexts.append({k:p.get(k,'UNKNOWN')for k in ['model','effort']})
  if r['type']=='response_item'and t in ['function_call','custom_tool_call']:calls.append({k:p.get(k)for k in ['name','call_id','arguments','input']}|{'timestamp':r.get('timestamp')})
  if r['type']=='response_item'and t in ['function_call_output','custom_tool_call_output']:results.append({k:p.get(k)for k in ['call_id','output']}|{'timestamp':r.get('timestamp')})
 return dict(session_id=session,parent_thread_id=parent,agent_path=spawn.get('agent_path'),source_session_sha256=hashlib.sha256(raw).hexdigest(),model_contexts=contexts,tool_calls_count=len(calls),tool_results_count=len(results),matched_call_ids=all(any(y['call_id']==x['call_id']for y in results)for x in calls),coverage='Recorded session response items only; no OS access trace. Probe is not student protocol certification.',**({}if probe else dict(tool_calls=calls,tool_results=results)))
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--source',type=Path,required=True);a.add_argument('--session',required=True);a.add_argument('--parent',required=True);a.add_argument('--output',type=Path,required=True);a.add_argument('--probe',action='store_true');v=a.parse_args();assert not v.output.exists();v.output.parent.mkdir(parents=True,exist_ok=True);v.output.write_text(json.dumps(export(v.source,v.session,v.parent,v.probe),ensure_ascii=False,indent=2)+'\n',encoding='utf8')
