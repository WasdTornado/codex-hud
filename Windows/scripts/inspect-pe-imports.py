"""仅 CI 失败诊断：列出本次测试二进制缺失的 DLL 导入，不访问账号。"""
import ctypes, pathlib, struct, sys
if sys.platform != 'win32':
    raise SystemExit('Windows only; not a passing test')
k=ctypes.WinDLL('kernel32',use_last_error=True)
k.LoadLibraryExW.argtypes=[ctypes.c_wchar_p,ctypes.c_void_p,ctypes.c_uint32];k.LoadLibraryExW.restype=ctypes.c_void_p
k.GetProcAddress.argtypes=[ctypes.c_void_p,ctypes.c_void_p];k.GetProcAddress.restype=ctypes.c_void_p
k.FreeLibrary.argtypes=[ctypes.c_void_p]
for path in pathlib.Path('Windows/src-tauri/target/debug/deps').glob('updates-*.exe'):
    data=path.read_bytes();pe=struct.unpack_from('<I',data,60)[0];optional=pe+24
    count=struct.unpack_from('<H',data,pe+6)[0];size=struct.unpack_from('<H',data,pe+20)[0]
    assert struct.unpack_from('<H',data,optional)[0]==0x20b,'Expected PE32+ x64'
    sections=[struct.unpack_from('<IIII',data,optional+size+i*40+8) for i in range(count)]
    def offset(rva):
        for virtual_size,virtual_address,raw_size,raw_offset in sections:
            if virtual_address<=rva<virtual_address+max(virtual_size,raw_size): return raw_offset+rva-virtual_address
        raise ValueError('Unmapped RVA')
    def string(rva):
        start=offset(rva);return data[start:data.index(b'\0',start)]
    desc=offset(struct.unpack_from('<I',data,optional+120)[0]);missing=[]
    while True:
        original,stamp,chain,name,first=struct.unpack_from('<IIIII',data,desc);desc+=20
        if not name:break
        dll=string(name).decode('ascii');handle=k.LoadLibraryExW(dll,None,1)
        if not handle:missing.append((dll,'DLL load failure',ctypes.get_last_error()));continue
        try:
            thunk=offset(original or first)
            while True:
                value=struct.unpack_from('<Q',data,thunk)[0];thunk+=8
                if not value:break
                if value&(1<<63): label='ordinal '+str(value&65535);ptr=ctypes.c_void_p(value&65535)
                else: label=string(value+2).decode('ascii');buf=ctypes.create_string_buffer(label.encode());ptr=ctypes.cast(buf,ctypes.c_void_p)
                if not k.GetProcAddress(handle,ptr):missing.append((dll,label,ctypes.get_last_error()))
        finally:k.FreeLibrary(handle)
    print(path.name,'missing imports:',missing,flush=True)
