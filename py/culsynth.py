from ctypes import *
import sys

_is_windows = sys.platform.startswith('win')

if _is_windows:
    _lib = CDLL('../target/debug/culsynth.dll')
else:
    _lib = CDLL('../target/debug/libculsynth.so')

_culsynth_get_sr_480 = _lib._culsynth_get_sr_480
_culsynth_get_sr_480.argtypes = []
_culsynth_get_sr_480.restype = c_uint32

_culsynth_get_sr_441 = _lib._culsynth_get_sr_441
_culsynth_get_sr_441.argtypes = []
_culsynth_get_sr_441.restype = c_uint32

_culsynth_osc_i16_new = _lib.culsynth_osc_i16_new
_culsynth_osc_i16_new.argtypes = []
_culsynth_osc_i16_new.restype = c_void_p

_culsynth_osc_i16_free = _lib.culsynth_osc_i16_free
_culsynth_osc_i16_free.argtypes = [c_void_p]
_culsynth_osc_i16_free.restype = None

_culsynth_osc_i16_process = _lib.culsynth_osc_i16_process
_culsynth_osc_i16_process.argtypes = [
    c_void_p,
    c_uint32,
    c_uint32,
    POINTER(c_uint16),
    POINTER(c_int16),
    POINTER(c_uint16),
    POINTER(c_int16),
    POINTER(c_int16),
    POINTER(c_int16),
    POINTER(c_int16)
]
_culsynth_osc_i16_process.restype = c_int32

class OscFxP:
    def __init__(self):
        self.ptr = _culsynth_osc_i16_new()
    def __del__(self):
        _culsynth_osc_i16_free(self.ptr)
    def process(self, note, tune, shape):
        num_samples = min(len(note), len(tune), len(shape))
        tri = (c_int16 * num_samples)()
        sq = (c_int16 * num_samples)()
        sn = (c_int16 * num_samples)()
        saw = (c_int16 * num_samples)()
        note_arr = (c_uint16*num_samples)(*note)
        tune_arr = (c_int16*num_samples)(*tune)
        shape_arr = (c_uint16*num_samples)(*shape)
        proc = _culsynth_osc_i16_process(self.ptr, _culsynth_get_sr_480(),
            num_samples, note_arr, tune_arr, shape_arr, sn, tri, sq, saw)
        return (sn, sq, tri, saw)

_culsynth_env_i16_new = _lib.culsynth_env_i16_new
_culsynth_env_i16_new.argtypes = []
_culsynth_env_i16_new.restype = c_void_p

_culsynth_env_i16_free = _lib.culsynth_env_i16_free
_culsynth_env_i16_free.argtypes = [c_void_p]
_culsynth_env_i16_free.restype = None

_culsynth_env_i16_process = _lib.culsynth_env_i16_process
_culsynth_env_i16_process.argtypes = [
    c_void_p,
    c_uint32,
    c_uint32,
    POINTER(c_uint8),
    POINTER(c_uint16),
    POINTER(c_uint16),
    POINTER(c_uint16),
    POINTER(c_uint16),
    POINTER(c_uint16)
]
_culsynth_env_i16_process.restype = c_int32

class EnvFxP:
    def __init__(self):
        self.ptr = _culsynth_env_i16_new()
    def __del__(self):
        _culsynth_env_i16_free(self.ptr)
    def process(self, gate, attack, decay, sustain, release):
        num_samples = min(len(gate), len(attack), len(decay), len(sustain), len(release))
        out = (c_int16 * num_samples)()
        gate_arr = (c_uint8*num_samples)(*gate)
        attack_arr = (c_uint16*num_samples)(*attack)
        decay_arr = (c_uint16*num_samples)(*decay)
        sustain_arr = (c_uint16*num_samples)(*sustain)
        release_arr = (c_uint16*num_samples)(*release)
        proc = _culsynth_env_i16_process(self.ptr, _culsynth_get_sr_480(),
            num_samples, gate_arr, attack_arr, decay_arr, sustain_arr, release_arr, out)
        return out
    
_culsynth_filt_i16_new = _lib.culsynth_filt_i16_new
_culsynth_filt_i16_new.argtypes = []
_culsynth_filt_i16_new.restype = c_void_p

_culsynth_filt_i16_free = _lib.culsynth_filt_i16_free
_culsynth_filt_i16_free.argtypes = [c_void_p]
_culsynth_filt_i16_free.restype = None

_culsynth_filt_i16_process = _lib.culsynth_filt_i16_process
_culsynth_filt_i16_process.argtypes = [
    c_void_p,
    c_uint32,
    c_uint32,
    POINTER(c_int16),
    POINTER(c_uint16),
    POINTER(c_uint16),
    POINTER(c_int16),
    POINTER(c_int16),
    POINTER(c_int16),
]
_culsynth_filt_i16_process.restype = c_int32

class FiltFxP:
    def __init__(self):
        self.ptr = _culsynth_filt_i16_new()
    def __del__(self):
        _culsynth_filt_i16_free(self.ptr)
    def process(self, input, cutoff, resonance):
        num_samples = min(len(x) for x in [input, cutoff, resonance])
        low = (c_int16*num_samples)()
        band = (c_int16*num_samples)()
        high = (c_int16*num_samples)()
        input_arr = (c_int16*num_samples)(*input)
        cutoff_arr = (c_uint16*num_samples)(*cutoff)
        resonance_arr = (c_uint16*num_samples)(*resonance)
        proc = _culsynth_filt_i16_process(self.ptr, _culsynth_get_sr_480(),
            num_samples, input_arr, cutoff_arr, resonance_arr, low, band, high)
        return (low, band, high)
