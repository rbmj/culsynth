from ctypes import *

lib = CDLL('../target/debug/janus.dll')

_janus_osc_u16_new = lib.janus_osc_u16_new
_janus_osc_u16_new.argtypes = []
_janus_osc_u16_new.restype = c_void_p

_janus_osc_u16_free = lib.janus_osc_u16_free
_janus_osc_u16_free.argtypes = [c_void_p]
_janus_osc_u16_free.restype = None

_janus_osc_u16_process = lib.janus_osc_u16_process
_janus_osc_u16_process.argtypes = [
    c_void_p,
    c_uint32,
    POINTER(c_uint16),
    POINTER(c_uint16),
    POINTER(POINTER(c_int16)),
    POINTER(POINTER(c_int16)),
    POINTER(POINTER(c_int16)),
    c_uint32
]
_janus_osc_u16_process.restype = c_int32

def _make_arr(x, type, num_samples):
    if not isinstance(x, list):
        x = [x]*num_samples
    elif len(x) != num_samples:
        raise IndexError
    return (type*num_samples)(*x)

def test_oscillator(num_samples, note, shape):
    osc_ptr = _janus_osc_u16_new()
    ns = c_uint32(num_samples)
    tri = POINTER(c_int16)()
    sq = POINTER(c_int16)()
    sn = POINTER(c_int16)()
    note_arr = (c_uint16*num_samples)(*([note]*num_samples))
    shape_arr = (c_uint16*num_samples)(*([shape]*num_samples))
    processed = 0
    sn_list = []
    sq_list = []
    tri_list = []
    while processed < num_samples:
        iter_proc = _janus_osc_u16_process(osc_ptr, ns, note_arr, shape_arr,
            byref(sn), byref(tri), byref(sq), processed)
        sn_list = sn_list + sn[:iter_proc]
        sq_list = sq_list + sq[:iter_proc]
        tri_list = tri_list + tri[:iter_proc]
        processed += iter_proc
    _janus_osc_u16_free(osc_ptr)
    return (sn_list, sq_list, tri_list)