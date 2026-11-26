"""Independently decode the 26.3 chunk packets used by the map regression test."""
import io
import struct
from check_party import integer

def values(f, count):
    bits = f.read(1)[0]
    if not bits:
        return [integer(f)] * count
    palette = [integer(f) for _ in range(integer(f))]
    out = []
    while len(out) < count:
        word, = struct.unpack('>Q', f.read(8))
        for _ in range(min(64 // bits, count - len(out))):
            out.append(palette[word & ((1 << bits) - 1)])
            word >>= bits
    return out

def unpack_chunk(frame):
    f = io.BytesIO(frame)
    cx, cz = struct.unpack('>ii', f.read(8))
    for _ in range(integer(f)):
        integer(f)
        f.read(8 * integer(f))
    data = io.BytesIO(f.read(integer(f)))
    course = {}
    for idx in range(24):
        solid, fluid = struct.unpack('>HH', data.read(4))
        blocks = values(data, 4096)
        values(data, 64)
        assert sum(b != 0 for b in blocks) == solid
        assert blocks.count(89) == fluid
        if idx >= 9 and solid:
            for i, state in enumerate(blocks):
                if state:
                    course[cx*16+(i&15),idx*16-64+(i>>8),cz*16+((i>>4)&15)] = state
    assert not data.read(1)
    assert integer(f) == 0
    masks = [int.from_bytes(f.read(integer(f)), 'little') for _ in range(4)]
    assert not(masks[0]&masks[2] or masks[1]&masks[3])
    for mask in masks[:2]:
        count = integer(f)
        assert count == mask.bit_count()
        for _ in range(count):
            assert integer(f) == 2048 and len(f.read(2048)) == 2048
    assert not f.read(1)
    return course

def verify_world(frames, rally):
    assert set(frames) == {(x,z) for x in range(-3,5) for z in range(-3,4)}
    terrain = {}
    for frame in frames.values():
        terrain.update(unpack_chunk(frame))
    if not rally:
        assert not terrain
        return
    for x in range(-30,50):
        for z in range(-30,37):
            assert (x,86 if -4<=x<21 else 88,z) in terrain
    for x,z in [(-22,16),(8,16),(8,-16),(39,-16),(39,27),(-5,27)]:
        assert terrain[x,94,z] == 14739
    assert (19,98,5) in terrain
