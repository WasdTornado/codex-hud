"""无账号 JSONL 假服务。只输出固定演示值，绝不读取登录文件。"""
import json, sys, time, subprocess
mode=sys.argv[1] if len(sys.argv)>1 else 'success'
initialized=False
initialize_id=1.0 if mode=='float-id' else 1
quota_id=2.0 if mode=='float-id' else 2
def send(value): print(json.dumps(value),flush=True)
for line in sys.stdin:
    request=json.loads(line)
    method=request.get('method')
    if method=='initialize':
        if mode=='eof': sys.exit(0)
        if mode=='timeout': time.sleep(60)
        if mode=='oversize': print('x'*1048576,flush=True); continue
        if mode=='total': print(('x'*1024+'\n')*4097,flush=True); continue
        if mode=='child':
            process=subprocess.Popen([sys.executable,'-c','import time;time.sleep(60)'])
            with open(sys.argv[2],'w') as output: output.write(str(process.pid))
            time.sleep(60)
        send({'id':2,'result':{'rateLimits':{}}})  # 乱序响应不应被接受。
        send({'id':'1','result':{}})  # 字符串 id 不应完成初始化。
        send({'id':True,'result':{}})  # 布尔值不能当作数字 1。
        send({'id':initialize_id,'result':{}})
    elif method=='initialized': initialized=True
    elif method=='account/rateLimits/read':
        assert initialized, '必须先发送 initialized'
        if mode=='error': send({'id':2,'error':{'message':'secret-token-do-not-display'}}); continue
        print('diagnostic noise',flush=True)
        send({'method':'unrelated/notification','params':{}})
        send({'id':'server-request','method':'unsupported/approval','params':{}})
        response=json.loads(sys.stdin.readline())
        assert response['id']=='server-request' and response['error']['code']==-32601
        send({'id':quota_id,'result':{'rateLimits':{'primary':{'windowDurationMins':10080,'usedPercent':53,'resetsAt':9000},'secondary':{'windowDurationMins':300,'usedPercent':18,'resetsAt':2000}}}})
