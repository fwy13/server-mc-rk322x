"""Run locally against a fresh --game color,paint --max-players 50 server."""
import argparse
import io
import queue
import socket
import struct
import threading
import time
import uuid

def vi(n):
    out=bytearray()
    while n>=128:
        out.append(n%128+128)
        n//=128
    return out+bytes([n])
def integer(f):
    v=0
    for shift in range(0,35,7):
        b=f.read(1)
        if not b: raise EOFError
        v|=(b[0]&127)<<shift
        if b[0]<128:return v
    raise ValueError('VarInt')
def string(s):
    b=s.encode()
    return vi(len(b))+b
def label(data):
    f=io.BytesIO(data)
    assert f.read(2)==bytes([10,8])
    def field():
        raw = f.read(int.from_bytes(f.read(2), 'big'))
        return raw.decode('utf8', 'surrogatepass').encode('utf-16-be', 'surrogatepass').decode('utf-16-be')
    assert field()=='text'
    return field()

class Client:
    def __init__(self,i,port):
        self.q=queue.Queue()
        self.chat=queue.Queue()
        self.warps=queue.Queue()
        self.world={}

        self.s=socket.create_connection(('127.0.0.1',port),timeout=25)
        self.r=self.s.makefile('rb')
        self.lock=threading.Lock()
        self.send(0,vi(777)+string('localhost')+struct.pack('>H',port)+vi(2))
        self.send(0,string(f'Party{i}')+uuid.uuid4().bytes)
        assert self.recv()[0]==2
        self.send(3)
        while True:
            pid,b=self.recv()
            if pid==15:self.send(7,b)
            if pid==3:break
        self.send(3)
        assert self.recv()[0]==0x32
        self.tiles={}
        self.peers=0
        self.chunks=0
        self.board=False
        self.subtitle=False
        self.first=(i==0)
        threading.Thread(target=self.read,daemon=True).start()
    def send(self,i,b=b''):
        p=vi(i)+b
        with self.lock:self.s.sendall(vi(len(p))+p)
    def recv(self):
        n=integer(self.r)
        f=io.BytesIO(self.r.read(n))
        return integer(f),f.read()
    def read(self):
        try:
            while True:
                i,b=self.recv()
                if i==0x7c:self.chat.put(label(b))
                if i==0x2e:
                    self.chunks+=1
                    self.world[struct.unpack('>ii', b[:8])]=b
                if i==1:self.peers+=1
                if i==0x64:self.board=True
                if i==0x72:self.subtitle=True
                if i==0x74:self.q.put(label(b))
                if i==0x2d:self.send(0x1c,b)
                if i==0x49:
                    f=io.BytesIO(b)
                    k=integer(f)
                    xyz=f.read(24)
                    self.send(0,vi(k)+xyz+bytes(8))
                    self.warps.put(struct.unpack('>ddd',xyz))
                    self.send(0x2c)
                if i==8 and self.first:
                    pos=int.from_bytes(b[:8],'big')
                    x,z=pos>>38,(pos>>12)&0x3ffffff
                    if x>=1<<25:x-=1<<26
                    if z>=1<<25:z-=1<<26
                    if pos&4095==72:self.tiles[x,z]=integer(io.BytesIO(b[8:]))
        except (EOFError,OSError):
            self.q.put('disconnected')
    def wait(self,labels):
        end=time.monotonic()+25
        while True:
            s=self.q.get(timeout=max(.01,end-time.monotonic()))
            if s=='disconnected':raise RuntimeError(s)
            if s in labels:return s

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--port',type=int,default=25565)
    parser.add_argument('--players',type=int,choices=range(2,51),default=5,metavar='2..50')
    a=parser.parse_args()
    clients=[]
    try:
        clients=[Client(i,a.port) for i in range(a.players)]
        first=clients[0]
        colors={'ĐỎ':17025,'VÀNG':17015,'XANH LÁ':17016,'XANH DƯƠNG':17022,'TRẮNG':17011,'TÍM':17021}
        target=colors[first.wait(colors)]
        safe=next(k for k,v in first.tiles.items() if v==target)
        wrong=next(k for k,v in first.tiles.items() if v!=target)
        for i,c in enumerate(clients):
            assert c.chunks and c.board and c.subtitle and c.peers==a.players-1
            x,z=wrong if i else safe
            c.send(0x1e,struct.pack('>dddB',x+.5,73,z+.5,1))
        first.wait({'KẾT THÚC'})
        first.wait({'Đại chiến sơn sàn'})
        print(f'PASS: {a.players} clients, chunks, players, title/subtitle, sidebar, verdict and next game')
    finally:
        for c in clients:
            c.s.shutdown(socket.SHUT_RDWR)
            c.s.close()
