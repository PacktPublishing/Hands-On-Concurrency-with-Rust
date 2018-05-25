import ctypes
import random

length = 1000000
lzc = ctypes.cdll.LoadLibrary("target/release/libzero_count.dylib")
arr = (ctypes.c_uint16 * length)(*[random.randint(0, 65000) for _ in range(0, length)])
print(lzc.tail_zero_count(ctypes.pointer(arr), length))
