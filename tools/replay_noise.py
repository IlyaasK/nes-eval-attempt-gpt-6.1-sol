"""Build an original NROM diagnostic that replays timestamped APU writes.

The default register set isolates noise. --registers 4011,4017 replays DAC
markers, and --offset permits measuring reference write timing in CPU cycles.
READ rows capture a CPU register read through two DAC levels. Every
generated instruction and branch-page penalty is accounted for.
"""
import argparse
import pathlib

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('trace')
parser.add_argument('rom')
parser.add_argument('--registers', default='400c,400e,400f,4015,4017')
parser.add_argument('--max-cycles', type=int, default=20_000_000)
parser.add_argument('--offset', type=int, default=0)
args = parser.parse_args()
registers = {int(a, 16) for a in args.registers.split(',')}
rows = []
for line in pathlib.Path(args.trace).read_text().splitlines():
    words = line.split()
    if not words or words[0] not in ['WRITE','READ']:
        continue
    kind,cycle,addr = words[:3]
    cycle,addr = int(cycle),int(addr,16)
    value = int(words[3]) if kind=='WRITE' else None
    if addr in registers and cycle < args.max_cycles:
        rows.append((cycle + (args.offset if cycle > 10000 else 0),addr,value))

code = bytearray([0x78])  # SEI; first fetch follows the eight reset cycles.
clock = 10
accumulator = None


def emit(data, cycles):
    global clock
    code.extend(data)
    clock += cycles


def delay(n):
    # A nested Y/X loop gives long delays without filling the 32 KiB PRG.
    while n > 2000:
        if (len(code) & 255) > 245:
            emit([0xea], 2)
            n -= 2
            continue
        count = min(255, (n - 1) // 1281)
        if n - (1281 * count + 1) == 1:
            count -= 1
        if not count:
            break
        cost = 1281 * count + 1
        emit([0xa0, count, 0xa2, 255, 0xca, 0xd0, 0xfd, 0x88, 0xd0, 0xf8], cost)
        n -= cost
    while n > 3:
        if n >= 6 and (len(code) & 255) in [251, 252, 253]:
            emit([0x24, 0], 3)
            n -= 3
            continue
        count = min(255, (n - 1) // 5)
        if count == 0:
            break
        cost = 5 * count + 1
        if n - cost == 1:
            count -= 1
            cost = 5 * count + 1
        if count == 0:
            break
        emit([0xa2, count, 0xca, 0xd0, 0xfd], cost)
        n -= cost
    while n >= 2:
        if n & 1:
            emit([0x24, 0], 3)
            n -= 3
        else:
            emit([0xea], 2)
            n -= 2
    assert n == 0, n


initialized = False
for target, addr, value in rows:
    if target > 10000 and not initialized:
        # A zero-page value lookup gives a three-cycle load when a two-cycle
        # immediate load would leave an impossible one-cycle delay.
        for v in range(256):
            emit([0xa9, v, 0x85, v], 5)
        accumulator = 255
        initialized = True
    if value is None:
        left = target-clock-4
        assert left>=0 and left!=1, (target,clock,'read requires a representable delay')
        delay(left)
        emit([0xad,addr&255,addr>>8],4)
        assert clock==target
        # Two DAC levels expose all eight input bits without disturbing
        # PPU memory. The second level includes the status register's D7.
        emit([0x8d,0x11,0x40],4)
        emit([0x4a],2)
        emit([0x8d,0x11,0x40],4)
        accumulator = None
        continue
    load = [] if accumulator == value else [0xa9, value]
    required = 4 + (2 if load else 0)
    left = target - clock - required
    if left == 1:
        assert initialized
        load = [0xa5, value]
        required = 7
        left = target - clock - required
    assert left >= 0, (target, clock)
    delay(left)
    if load:
        emit(load, 3 if load[0] == 0xa5 else 2)
        accumulator = value
    emit([0x8d, addr & 255, addr >> 8], 4)
    assert clock == target
assert len(code) < 32762, len(code)
end = 0x8000 + len(code)
emit([0x4c, end & 255, end >> 8], 3)
prg = code + bytes(32762 - len(code)) + bytes([
    end & 255, end >> 8, 0, 0x80, end & 255, end >> 8])
rom = b'NES\x1a' + bytes([2, 0, 0, 0]) + bytes(8) + prg
pathlib.Path(args.rom).write_bytes(rom)
print('bytes', len(code), 'cycles', clock)
