from ctypes import *

_lib = CDLL('../target/debug/janus.dll')

_janus_osc_u16_new = _lib.janus_osc_u16_new
_janus_osc_u16_new.argtypes = []
_janus_osc_u16_new.restype = c_void_p

_janus_osc_u16_free = _lib.janus_osc_u16_free
_janus_osc_u16_free.argtypes = [c_void_p]
_janus_osc_u16_free.restype = None

_janus_osc_u16_process = _lib.janus_osc_u16_process
_janus_osc_u16_process.argtypes = [
    c_void_p,
    c_uint32,
    POINTER(c_uint16),
    POINTER(c_uint16),
    POINTER(POINTER(c_int16)),
    POINTER(POINTER(c_int16)),
    POINTER(POINTER(c_int16)),
    POINTER(POINTER(c_int16)),
    c_uint32
]
_janus_osc_u16_process.restype = c_int32

class OscFxP:
    def __init__(self):
        self.ptr = _janus_osc_u16_new()
    def __del__(self):
        _janus_osc_u16_free(self.ptr)
    def process(self, note, shape):
        num_samples = min(len(note), len(shape))
        tri = POINTER(c_int16)()
        sq = POINTER(c_int16)()
        sn = POINTER(c_int16)()
        saw = POINTER(c_int16)()
        note_arr = (c_uint16*num_samples)(*note)
        shape_arr = (c_uint16*num_samples)(*shape)
        processed = 0
        sn_list = []
        sq_list = []
        tri_list = []
        saw_list = []
        while processed < num_samples:
            iter_proc = _janus_osc_u16_process(self.ptr,
                c_uint32(num_samples - processed), note_arr, shape_arr,
                byref(sn), byref(tri), byref(sq), byref(saw), c_uint32(processed))
            sn_list = sn_list + sn[:iter_proc]
            sq_list = sq_list + sq[:iter_proc]
            tri_list = tri_list + tri[:iter_proc]
            saw_list = saw_list + saw[:iter_proc]
            processed += iter_proc
        return (sn_list, sq_list, tri_list, saw_list)

_janus_env_u16_new = _lib.janus_env_u16_new
_janus_env_u16_new.argtypes = []
_janus_env_u16_new.restype = c_void_p

_janus_env_u16_free = _lib.janus_env_u16_free
_janus_env_u16_free.argtypes = [c_void_p]
_janus_env_u16_free.restype = None

_janus_env_u16_process = _lib.janus_env_u16_process
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
    
_janus_filt_u16_new = _lib.janus_filt_u16_new
_janus_filt_u16_new.argtypes = []
_janus_filt_u16_new.restype = c_void_p

_janus_filt_u16_free = _lib.janus_filt_u16_free
_janus_filt_u16_free.argtypes = [c_void_p]
_janus_filt_u16_free.restype = None

_janus_filt_u16_process = _lib.janus_filt_u16_process
_janus_filt_u16_process.argtypes = [
    c_void_p,
    c_uint32,
    POINTER(c_int16),
    POINTER(c_uint16),
    POINTER(c_uint16),
    POINTER(POINTER(c_int16)),
    POINTER(POINTER(c_int16)),
    POINTER(POINTER(c_int16)),
    c_uint32
]
_janus_filt_u16_process.restype = c_int32

class FiltFxP:
    def __init__(self):
        self.ptr = _janus_filt_u16_new()
    def __del__(self):
        _janus_filt_u16_free(self.ptr)
    def process(self, input, cutoff, resonance):
        num_samples = min(len(x) for x in [input, cutoff, resonance])
        low = POINTER(c_int16)()
        band = POINTER(c_int16)()
        high = POINTER(c_int16)()
        input_arr = (c_int16*num_samples)(*input)
        cutoff_arr = (c_uint16*num_samples)(*cutoff)
        resonance_arr = (c_uint16*num_samples)(*resonance)
        processed = 0
        low_list = []
        band_list = []
        high_list = []
        while processed < num_samples:
            iter_proc = _janus_filt_u16_process(self.ptr,
                c_uint32(num_samples - processed), input_arr, cutoff_arr,
                resonance_arr, byref(low), byref(band), byref(high),
                c_uint32(processed))
            low_list = low_list + low[:iter_proc]
            band_list = band_list + band[:iter_proc]
            high_list = high_list + high[:iter_proc]
            processed += iter_proc
        return (low_list, band_list, high_list)
    

_janus_osc_f32_new = _lib.janus_osc_f32_new
_janus_osc_f32_new.argtypes = []
_janus_osc_f32_new.restype = c_void_p

_janus_osc_f32_free = _lib.janus_osc_f32_free
_janus_osc_f32_free.argtypes = [c_void_p]
_janus_osc_f32_free.restype = None

_janus_osc_f32_process = _lib.janus_osc_f32_process
_janus_osc_f32_process.argtypes = [
    c_void_p,
    c_uint32,
    POINTER(c_float),
    POINTER(c_float),
    POINTER(POINTER(c_float)),
    POINTER(POINTER(c_float)),
    POINTER(POINTER(c_float)),
    POINTER(POINTER(c_float)),
    c_uint32
]
_janus_osc_f32_process.restype = c_int32

class OscFloat:
    def __init__(self):
        self.ptr = _janus_osc_f32_new()
    def __del__(self):
        _janus_osc_f32_free(self.ptr)
    def process(self, note, shape):
        num_samples = min(len(note), len(shape))
        tri = POINTER(c_float)()
        sq = POINTER(c_float)()
        sn = POINTER(c_float)()
        saw = POINTER(c_float)()
        note_arr = (c_float*num_samples)(*note)
        shape_arr = (c_float*num_samples)(*shape)
        processed = 0
        sn_list = []
        sq_list = []
        tri_list = []
        saw_list = []
        while processed < num_samples:
            iter_proc = _janus_osc_f32_process(self.ptr,
                c_uint32(num_samples - processed), note_arr, shape_arr,
                byref(sn), byref(tri), byref(sq), byref(saw), c_uint32(processed))
            sn_list = sn_list + sn[:iter_proc]
            sq_list = sq_list + sq[:iter_proc]
            tri_list = tri_list + tri[:iter_proc]
            saw_list = saw_list + saw[:iter_proc]
            processed += iter_proc
        return (sn_list, sq_list, tri_list, saw_list)

_janus_env_f32_new = _lib.janus_env_f32_new
_janus_env_f32_new.argtypes = []
_janus_env_f32_new.restype = c_void_p

_janus_env_f32_free = _lib.janus_env_f32_free
_janus_env_f32_free.argtypes = [c_void_p]
_janus_env_f32_free.restype = None

_janus_env_f32_process = _lib.janus_env_f32_process
_janus_env_f32_process.argtypes = [
    c_void_p,
    c_uint32,
    POINTER(c_float),
    POINTER(c_float),
    POINTER(c_float),
    POINTER(c_float),
    POINTER(c_float),
    POINTER(POINTER(c_float)),
    c_uint32
]
_janus_env_f32_process.restype = c_int32

class EnvFloat:
    def __init__(self):
        self.ptr = _janus_env_f32_new()
    def __del__(self):
        _janus_env_f32_free(self.ptr)
    def process(self, gate, attack, decay, sustain, release):
        num_samples = min(len(x) for x in [gate, attack, decay, sustain, release])
        signal = POINTER(c_float)()
        gate_arr = (c_float*num_samples)(*gate)
        attack_arr = (c_float*num_samples)(*attack)
        decay_arr = (c_float*num_samples)(*decay)
        sustain_arr = (c_float*num_samples)(*sustain)
        release_arr = (c_float*num_samples)(*release)
        processed = 0
        output = []
        while processed < num_samples:
            iter_proc = _janus_env_f32_process(self.ptr,
                c_uint32(num_samples - processed), gate_arr, attack_arr,
                decay_arr, sustain_arr, release_arr, byref(signal),
                c_uint32(processed))
            output = output + signal[:iter_proc]
            processed += iter_proc
        return output
    
_janus_filt_f32_new = _lib.janus_filt_f32_new
_janus_filt_f32_new.argtypes = []
_janus_filt_f32_new.restype = c_void_p

_janus_filt_f32_free = _lib.janus_filt_f32_free
_janus_filt_f32_free.argtypes = [c_void_p]
_janus_filt_f32_free.restype = None

_janus_filt_f32_process = _lib.janus_filt_f32_process
_janus_filt_f32_process.argtypes = [
    c_void_p,
    c_uint32,
    POINTER(c_float),
    POINTER(c_float),
    POINTER(c_float),
    POINTER(POINTER(c_float)),
    POINTER(POINTER(c_float)),
    POINTER(POINTER(c_float)),
    c_uint32
]
_janus_filt_f32_process.restype = c_int32

class FiltFloat:
    def __init__(self):
        self.ptr = _janus_filt_f32_new()
    def __del__(self):
        _janus_filt_f32_free(self.ptr)
    def process(self, input, cutoff, resonance):
        num_samples = min(len(x) for x in [input, cutoff, resonance])
        low = POINTER(c_float)()
        band = POINTER(c_float)()
        high = POINTER(c_float)()
        input_arr = (c_float*num_samples)(*input)
        cutoff_arr = (c_float*num_samples)(*cutoff)
        resonance_arr = (c_float*num_samples)(*resonance)
        processed = 0
        low_list = []
        band_list = []
        high_list = []
        while processed < num_samples:
            iter_proc = _janus_filt_f32_process(self.ptr,
                c_uint32(num_samples - processed), input_arr, cutoff_arr,
                resonance_arr, byref(low), byref(band), byref(high),
                c_uint32(processed))
            low_list = low_list + low[:iter_proc]
            band_list = band_list + band[:iter_proc]
            high_list = high_list + high[:iter_proc]
            processed += iter_proc
        return (low_list, band_list, high_list)