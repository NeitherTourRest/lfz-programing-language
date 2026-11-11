"""公开自调服务，仅公开协议，不含隐藏验收。启动后打印临时baseURL，CtrlC关闭。"""
import http.server
class Handler(http.server.BaseHTTPRequestHandler):
 def log_message(self,*args):pass
 def do_GET(self):
  data={'/ok':'雪'.encode(),'/missing':b'absent','/redirect':b'','/bad':b'\xff'};body=data.get(self.path,b'no');self.send_response({'/missing':404,'/redirect':302}.get(self.path,200));self.send_header('Content-Length',str(len(body)))
  if self.path=='/ok':self.send_header('X-Repeat','one');self.send_header('X-Repeat','two')
  if self.path=='/redirect':self.send_header('Location','/target')
  self.end_headers();self.wfile.write(body)
if __name__=='__main__':
 s=http.server.ThreadingHTTPServer(('127.0.0.1',0),Handler);print('http://127.0.0.1:'+str(s.server_port),flush=True)
 try:s.serve_forever()
 except KeyboardInterrupt:pass
 finally:s.server_close()
