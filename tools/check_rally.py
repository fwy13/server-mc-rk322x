"""Run against an empty --game rally,paint server on localhost."""
import argparse
import socket
import struct
import time
from check_party import Client, string
from map_check import verify_world

def until(q, pred):
    end=time.monotonic()+24
    while True:
        item=q.get(timeout=max(.01,end-time.monotonic()))
        if pred(item):return item

def say(client, message):
    stamp=int(time.time()*1000)
    client.send(9,string(message)+struct.pack('>qq',stamp,0)+bytes(6))

def move(client,x,y,z):
    client.send(0x1e,struct.pack('>dddB',x,y,z,1))
    time.sleep(.15)

def main(port):
    a,b=Client(0,port),Client(1,port)
    clients=[a,b]
    try:
        for sender,who,msg in [(a,'Party0','Chào bạn ⛄'),(b,'Party1','Đua nhé 🏁')]:
            say(sender,msg)
            for c in clients:until(c.chat,lambda s:s==f'<{who}> {msg}')
        until(a.warps,lambda xyz:xyz[1]==89)
        verify_world(a.world.copy(), True)
        late=Client(2,port)
        clients.append(late)
        until(late.warps,lambda xyz:xyz[1]==99)
        assert late.world==a.world
        say(late,'Mình đang ở khán đài!')
        for c in clients:until(c.chat,lambda s:s=='<Party2> Mình đang ở khán đài!')
        # Touching the finish early must not count. Falling still returns to start.
        move(a,-4.5,89,27.5)
        move(a,8,85,6)
        until(a.warps,lambda xyz:xyz==(-23.5,89,-19.5))
        move(a,-21.5,89,16.5)
        move(a,8,85,6)
        until(a.warps,lambda xyz:xyz==(-21.5,89,16.5))
        for x,z in [(8,16),(8,-16),(39,-16),(39,27),(-5,27)]:move(a,x+.5,89,z+.5)
        for c in clients:
            until(c.chat,lambda s:'Party0 chiến thắng!' in s)
            c.wait({'Đại chiến sơn sàn'})
            verify_world(c.world.copy(), False)
        print('PASS: two-way Unicode chat, spectator, map sync/reset, ordered gates, falls, winner, next game')
    finally:
        for c in clients:
            c.s.shutdown(socket.SHUT_RDWR)
            c.s.close()

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--port',type=int,default=25565)
    main(parser.parse_args().port)
