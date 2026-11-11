from pathlib import Path
import threading,runpy,sys
original=threading.Thread.join
def inject_after_real_join(self,*args,**kwargs):
 target=getattr(self,'_target',None)
 server_thread=getattr(target,'__name__','')=='serve_forever'
 original(self,*args,**kwargs)
 if server_thread:
  assert not self.is_alive()
  self.is_alive=lambda: True
threading.Thread.join=inject_after_real_join
runner=sys.argv.pop(1)
sys.path.insert(0,str(Path(runner).resolve().parent))
runpy.run_path(runner,run_name='__main__')
