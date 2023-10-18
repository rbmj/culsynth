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

class OscillatorFxP:
    def __init__(self):
        self.ptr = _janus_osc_u16_new()
    def __del__(self):
        _janus_osc_u16_free(self.ptr)
    def process(self, note, shape):
        num_samples = min(len(note), len(shape))
        tri = POINTER(c_int16)()
        sq = POINTER(c_int16)()
        sn = POINTER(c_int16)()
        note_arr = (c_uint16*num_samples)(*note)
        shape_arr = (c_uint16*num_samples)(*shape)
        processed = 0
        sn_list = []
        sq_list = []
        tri_list = []
        while processed < num_samples:
            iter_proc = _janus_osc_u16_process(self.ptr,
                c_uint32(num_samples - processed), note_arr, shape_arr,
                byref(sn), byref(tri), byref(sq), c_uint32(processed))
            sn_list = sn_list + sn[:iter_proc]
            sq_list = sq_list + sq[:iter_proc]
            tri_list = tri_list + tri[:iter_proc]
            processed += iter_proc
        return (sn_list, sq_list, tri_list)

_janus_env_u16_new = lib.janus_env_u16_new
_janus_env_u16_new.argtypes = []
_janus_env_u16_new.restype = c_void_p

_janus_env_u16_free = lib.janus_env_u16_free
_janus_env_u16_free.argtypes = [c_void_p]
_janus_env_u16_free.restype = None

_janus_env_u16_process = lib.janus_env_u16_process
_janus_env_u16_process.argtypes = [
    c_void_p,
    c_uint32,
    POINTER(c_int16),
    POINTER(c_uint16),
    POINTER(c_uint16),
    POINTER(c_uint16),
    POINTER(c_uint16),
    POINTER(POINTER(c_uint16)),
    c_uint32
]
_janus_env_u16_process.restype = c_int32

class EnvFxP:
    def __init__(self):
        self.ptr = _janus_env_u16_new()
    def __del__(self):
        _janus_env_u16_free(self.ptr)
    def process(self, gate, attack, decay, sustain, release):
        num_samples = min(len(x) for x in [gate, attack, decay, sustain, release])
        signal = POINTER(c_uint16)()
        gate_arr = (c_int16*num_samples)(*gate)
        attack_arr = (c_uint16*num_samples)(*attack)
        decay_arr = (c_uint16*num_samples)(*decay)
        sustain_arr = (c_uint16*num_samples)(*sustain)
        release_arr = (c_uint16*num_samples)(*release)
        processed = 0
        output = []
        while processed < num_samples:
            iter_proc = _janus_env_u16_process(self.ptr,
                c_uint32(num_samples - processed), gate_arr, attack_arr,
                decay_arr, sustain_arr, release_arr, byref(signal),
                c_uint32(processed))
            output = output + signal[:iter_proc]
            processed += iter_proc
        return output