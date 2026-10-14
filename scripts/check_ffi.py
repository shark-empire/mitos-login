#!/usr/bin/env python3
"""Cross-checks src/sys/ffi.rs against the real system headers.

Usage: check_ffi.py C_OUTPUT.txt path/to/ffi.rs
The C output comes from scripts/ffi_check.c. Constants are compared by value;
struct layouts are rebuilt with ctypes from the Rust field lists and compared
with the C compiler's sizeof/offsetof.
"""
import ctypes, re, sys

c_out = {}
for line in open(sys.argv[1]):
    k, v = line.strip().split("=", 1)
    c_out[k] = int(v)
src = open(sys.argv[2]).read()

errors = 0
def fail(msg):
    global errors
    errors += 1
    print("MISMATCH:", msg)

# ---- constants
consts = dict((m.group(1), m.group(3)) for m in re.finditer(r"pub const (\w+): (\w+) = ([^;]+);", src))
def pyval(s):
    s = s.strip().replace("_", "")
    return int(s, 0)
checked = 0
for name, lit in consts.items():
    if name in ("LFLAG_INDEX",):
        continue
    if name not in c_out:
        fail("constant %s has no C counterpart" % name)
        continue
    try:
        v = pyval(lit)
    except ValueError:
        fail("cannot evaluate %s = %s" % (name, lit))
        continue
    if v != c_out[name]:
        fail("%s: rust=%d c=%d" % (name, v, c_out[name]))
    checked += 1
print("constants checked:", checked)

# ---- structs
TYPES = {
    "u8": ctypes.c_uint8, "u16": ctypes.c_uint16, "u32": ctypes.c_uint32, "u64": ctypes.c_uint64,
    "i16": ctypes.c_int16, "i32": ctypes.c_int32, "i64": ctypes.c_int64,
    "c_int": ctypes.c_int, "c_ulong": ctypes.c_ulong, "usize": ctypes.c_size_t,
}
structs = {}
def ctype_of(t):
    t = t.strip()
    if t.startswith("*mut") or t.startswith("*const"):
        return ctypes.c_void_p
    m = re.match(r"\[(\w+); (\d+)\]", t)
    if m:
        return ctype_of(m.group(1)) * int(m.group(2))
    if t in TYPES:
        return TYPES[t]
    if t in structs:
        return structs[t]
    raise KeyError(t)

for m in re.finditer(r"#\[repr\(C\)\][^\n]*\n(?:#\[[^\n]*\n)*pub struct (\w+) \{(.*?)\n\}", src, re.S):
    name, body = m.group(1), m.group(2)
    fields = []
    for fm in re.finditer(r"pub (\w+): ([^,\n]+),", body):
        fields.append((fm.group(1), ctype_of(fm.group(2))))
    structs[name] = type(name, (ctypes.Structure,), {"_fields_": fields})

expect_size = {
    "PollFd": "sizeof_struct pollfd", "SigSet": "sizeof_sigset_t", "RLimit": "sizeof_struct rlimit",
    "UCred": "sizeof_struct ucred", "Passwd": "sizeof_struct passwd",
    "FbVarScreeninfo": "sizeof_struct fb_var_screeninfo", "FbFixScreeninfo": "sizeof_struct fb_fix_screeninfo",
    "VtStat": "sizeof_struct vt_stat", "InputEvent": "sizeof_struct input_event",
}
for rs, ck in expect_size.items():
    if rs not in structs:
        fail("struct %s not found in ffi.rs" % rs); continue
    got = ctypes.sizeof(structs[rs])
    if got != c_out[ck]:
        fail("sizeof %s: rust-layout=%d c=%d" % (rs, got, c_out[ck]))
expect_off = [
    ("Passwd", "pw_uid", "offsetof_struct passwd_pw_uid"), ("Passwd", "pw_gecos", "offsetof_struct passwd_pw_gecos"),
    ("Passwd", "pw_dir", "offsetof_struct passwd_pw_dir"), ("Passwd", "pw_shell", "offsetof_struct passwd_pw_shell"),
    ("FbVarScreeninfo", "bits_per_pixel", "offsetof_struct fb_var_screeninfo_bits_per_pixel"),
    ("FbVarScreeninfo", "red", "offsetof_struct fb_var_screeninfo_red"),
    ("FbVarScreeninfo", "height", "offsetof_struct fb_var_screeninfo_height"),
    ("FbVarScreeninfo", "width", "offsetof_struct fb_var_screeninfo_width"),
    ("FbVarScreeninfo", "reserved", "offsetof_struct fb_var_screeninfo_reserved"),
    ("FbFixScreeninfo", "smem_len", "offsetof_struct fb_fix_screeninfo_smem_len"),
    ("FbFixScreeninfo", "line_length", "offsetof_struct fb_fix_screeninfo_line_length"),
    ("FbFixScreeninfo", "mmio_start", "offsetof_struct fb_fix_screeninfo_mmio_start"),
    ("VtStat", "v_active", "offsetof_struct vt_stat_v_active"),
    ("VtStat", "v_signal", "offsetof_struct vt_stat_v_signal"),
    ("VtStat", "v_state", "offsetof_struct vt_stat_v_state"),
    ("InputEvent", "type_", "offsetof_struct input_event_type"),
    ("InputEvent", "code", "offsetof_struct input_event_code"),
    ("InputEvent", "value", "offsetof_struct input_event_value"),
]
for rs, field, ck in expect_off:
    got = getattr(structs[rs], field).offset
    if got != c_out[ck]:
        fail("offsetof %s.%s: rust-layout=%d c=%d" % (rs, field, got, c_out[ck]))

if c_out["sizeof_struct termios"] != 60 or c_out["offsetof_struct termios_c_lflag"] != 12:
    fail("termios buffer assumption (60 bytes, c_lflag at byte 12) is wrong")
if c_out["sizeof_struct input_event"] != 24 or c_out["offsetof_struct input_event_type"] != 16 \
        or c_out["offsetof_struct input_event_code"] != 18 or c_out["offsetof_struct input_event_value"] != 20:
    fail("input_event layout assumption is wrong")
if c_out["sizeof_struct signalfd_siginfo"] != 128:
    fail("signalfd_siginfo size assumption is wrong")

print("structs checked:", len(expect_size), "sizes,", len(expect_off), "offsets")
# ---- _IOC()-derived evdev ioctl numbers: same bit-packing formula as ffi.rs's
# ioc_read/eviocgname/eviocgbit, computed independently in Python and checked
# against the real macro expansions from the C probe.
def ioc_read(ty, nr, size):
    return ((2 << 30) | (ty << 8) | nr | (size << 16)) & 0xffffffff
def eviocgname(l): return ioc_read(ord('E'), 0x06, l)
def eviocgbit(ev, l): return ioc_read(ord('E'), 0x20 + ev, l)
ioc_checks = [
    ("EVIOCGNAME_32", eviocgname(32)), ("EVIOCGNAME_256", eviocgname(256)),
    ("EVIOCGBIT_0_32", eviocgbit(0, 32)), ("EVIOCGBIT_EVKEY_96", eviocgbit(1, 96)),
    ("EVIOCGBIT_EVREL_8", eviocgbit(2, 8)),
]
for name, val in ioc_checks:
    if c_out[name] != val:
        fail("%s: formula=%d c=%d" % (name, val, c_out[name]))
print("ioc formula checks:", len(ioc_checks))

print("FFI CHECK:", "FAILED (%d)" % errors if errors else "ok")
sys.exit(1 if errors else 0)
