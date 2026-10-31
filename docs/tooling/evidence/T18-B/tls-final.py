from pathlib import Path
import ssl,socket,threading,subprocess,json,os
R=Path.cwd();W=R/'Temp/T18-B-tooling';ctx=ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER);ctx.load_cert_chain(W/'tls-cert.pem',W/'tls-key.pem');server=socket.socket();server.bind(('127.0.0.1',0));server.listen(1);server.settimeout(3);events=[]
def serve():
 try:
  conn,_=server.accept();conn.settimeout(3);events.append('accepted')
  try:
   with ctx.wrap_socket(conn,server_side=True) as tls:events.append('server-side TLS handshake complete; not client trust proof');events.append('application_bytes='+str(len(tls.recv(1024))))
  except ssl.SSLError as e:events.append(str(e))
  finally:conn.close()
 except Exception as e:events.append(repr(e))
 finally:server.close()
t=threading.Thread(target=serve);t.start();port=server.getsockname()[1];code=f'httpRequest("GET","https://127.0.0.1:{port}/",[],"",2000,4096)'
env=os.environ.copy();env.update(TEMP=str(R/'Temp/toolchain'),TMP=str(R/'Temp/toolchain'))
try:p=subprocess.run([str(R/'target/release/lfz.exe'),'--allow-http','-e',code,'--json'],cwd=R,env=env,capture_output=True,timeout=8)
finally:t.join(5)
assert not t.is_alive();d={'command':'lfz --allow-http -e <local self-signed HTTPS> --json','exit':p.returncode,'stdout':p.stdout.decode('utf-8'),'stderr':p.stderr.decode('utf-8'),'server_events':events,'trusted_tls_success_tested':False};(W/'tls-rejection-final.json').write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps(d,ensure_ascii=False));assert p.returncode==2 and b'IOError' in p.stdout and not p.stderr and events[0]=='accepted'
