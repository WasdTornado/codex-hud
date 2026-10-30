"""仅提供本机静态演示，不含真实账号数据或宿主接口。"""
import http.server
from pathlib import Path
from functools import partial
root=Path(__file__).resolve().parents[1]/'ui'
server=http.server.ThreadingHTTPServer(('127.0.0.1',4173),partial(http.server.SimpleHTTPRequestHandler,directory=str(root)))
print('演示值预览：http://127.0.0.1:4173；不是 Windows 验收',flush=True)
server.serve_forever()
