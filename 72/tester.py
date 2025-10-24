import subprocess, random
from operator import add, sub, mul, floordiv as quo, mod as rem

# Set DEBUG = 1 to print extra info if needed
DEBUG = 0

bigone, bigtwo = random.randint(2 ** 500, 2 ** 512), random.randint(2 ** 500, 2 ** 512)
hexone, hextwo = hex(bigone), hex(bigtwo)

if DEBUG:
    print("\nhexone =", hexone, "\nhextwo =", hextwo)

ops = {'ADD': add, 'SUB': sub, 'MUL': mul, 'QUO': quo, 'REM': rem}

for op in ops:
    result_hex = subprocess.check_output(["cargo", "run", hexone, hextwo, op]).decode().strip()
    result_int = int(result_hex, 16)
    expected = ops[op](bigone, bigtwo)

    if result_int != expected:
        print("Operator", op, "FAILED.")
        if DEBUG:
            print("Expected:", hex(expected))
            print("Received:", result_hex)
        exit()
    else:
        print(op, "passes.")
