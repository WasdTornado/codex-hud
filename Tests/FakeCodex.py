"""协议测试服务，不读取真实账号或网络。"""
import json, sys
mode = sys.argv[1] if len(sys.argv) > 1 else 'success'
for line in sys.stdin:
    request = json.loads(line)
    if request['method'] == 'initialize':
        print(json.dumps({'id': 1, 'result': {}}), flush=True)
    if request['method'] == 'account/rateLimits/read':
        if mode == 'timeout': continue
        if mode == 'error':
            print(json.dumps({'id': 2, 'error': {'message': 'secret-token-do-not-display'}}), flush=True)
            continue
        print('diagnostic noise', flush=True)
        print(json.dumps({'id': 2, 'result': {'rateLimits': {
            'primary': {'windowDurationMins': 10080, 'usedPercent': 53, 'resetsAt': 9000},
            'secondary': {'windowDurationMins': 300, 'usedPercent': 18, 'resetsAt': 2000}
        }}}), flush=True)
