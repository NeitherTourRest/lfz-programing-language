from pathlib import Path
import hashlib,json
p=Path('Temp/T18-B-tooling');raw=(p/'public-raw-headers.txt').read_bytes();wire=b'HTTP/1.1 200 Test\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok'
d={'raw_headers_crlf':raw.decode('utf-8'),'fixture_wire_sha256':hashlib.sha256(wire).hexdigest(),'header_utf8_bytes':len(raw),'body_bytes':2,'N':len(raw)+2,'native_utf16_bytes_excluding_NUL':len((p/'public-raw-headers.u16le').read_bytes())}
(p/'public-header-witness.json').write_text(json.dumps(d,indent=2)+'\n',encoding='utf-8');print(json.dumps(d))
