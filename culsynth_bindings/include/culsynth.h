#ifndef CULSYNTH_H_INC
#define CULSYNTH_H_INC
#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>

extern uint32_t CULSYNTH_SR_480;
extern uint32_t CULSYNTH_SR_441;

void* culsynth_amp_i16_new();
void culsynth_amp_i16_free(void*);
int32_t culsynth_amp_i16_process(
    void* amp,
    uint32_t sample_rate,
    uint32_t samples,
    const int16_t* signal,
    const uint16_t* gain,
    int16_t* out
);
void* culsynth_amp_f32_new();
void culsynth_amp_f32_free(void*);
int32_t culsynth_amp_f32_process(
    void* amp,
    uint32_t sample_rate,
    uint32_t samples,
    const float* signal,
    const float* gain,
    float* out
);

void* culsynth_env_i16_new();
void culsynth_env_i16_free(void*);
int32_t culsynth_env_i16_process(
    void* env,
    uint32_t sample_rate,
    uint32_t samples,
    const uint8_t* gate,
    const uint16_t* attack,
    const uint16_t* decay,
    const uint16_t* sustain,
    const uint16_t* release,
    uint16_t* signal
);
void* culsynth_env_f32_new();
void culsynth_env_f32_free(void*);
int32_t culsynth_env_f32_process(
    void* env,
    uint32_t sample_rate,
    uint32_t samples,
    const uint8_t* gate,
    const float* attack,
    const float* decay,
    const float* sustain,
    const float* release,
    float* signal
);

void* culsynth_filt_i16_new();
void culsynth_filt_i16_free(void*);
int32_t culsynth_filt_i16_process(
    void* filt,
    uint32_t sample_rate,
    uint32_t samples,
    const int16_t* input,
    const uint16_t* cutoff,
    const uint16_t* resonance,
    int16_t* low,
    int16_t* band,
    int16_t* high
);
void* culsynth_filt_f32_new();
void culsynth_filt_f32_free(void*);
int32_t culsynth_filt_f32_process(
    void* filt,
    uint32_t sample_rate,
    uint32_t samples,
    const float* input,
    const float* cutoff,
    const float* resonance,
    float* low,
    float* band,
    float* high
);

void* culsynth_osc_i16_new();
void culsynth_osc_i16_free(void*);
int32_t culsynth_osc_i16_process(
    void* osc,
    uint32_t sample_rate,
    uint32_t samples,
    const uint16_t* note,
    const int16_t* tune,
    const uint16_t* shape,
    int16_t* sin,
    int16_t* tri,
    int16_t* sq,
    int16_t* saw
);
void* culsynth_osc_f32_new();
void culsynth_osc_f32_free(void*);
int32_t culsynth_osc_f32_process(
    void* osc,
    uint32_t sample_rate,
    uint32_t samples,
    const float* note,
    const float* tune,
    const float* shape,
    float* sin,
    float* tri,
    float* sq,
    float* saw
);

#ifdef __cplusplus
}

namespace culsynth {
    class Amp {
        void* ffi;
        Amp(const Amp&);
        Amp& operator=(const Amp&);
    public:
        Amp() : ffi(culsynth_amp_f32_new()) {}
        ~Amp() { culsynth_amp_f32_free(ffi); }
        int32_t process(uint32_t sample_rate, uint32_t samples,
            const float* signal, const float* gain, float* out)
        {
            return culsynth_amp_f32_process(ffi, sample_rate, samples, signal,
                gain, out);
        }
    };

    class AmpFxP {
        void* ffi;
        AmpFxP(const AmpFxP&);
        AmpFxP& operator=(const AmpFxP&);
    public:
        AmpFxP() : ffi(culsynth_amp_i16_new()) {}
        ~AmpFxP() { culsynth_amp_f32_free(ffi); }
        int32_t process(uint32_t sample_rate, uint32_t samples,
            const int16_t* signal, const uint16_t* gain, int16_t* out)
        {
            return culsynth_amp_i16_process(ffi, sample_rate, samples, signal,
                gain, out);
        }
    };

    class Env {
        void* ffi;
        Env(const Env&);
        Env& operator=(const Env&);
    public:
        Env() : ffi(culsynth_env_f32_new()) {}
        ~Env() { culsynth_env_f32_free(ffi); }
        int32_t process(
            uint32_t sample_rate,
            uint32_t samples,
            const uint8_t* gate,
            const float* attack,
            const float* decay,
            const float* sustain,
            const float* release,
            float* signal)
        {
            return culsynth_env_f32_process(ffi, sample_rate, samples, gate,
                attack, decay, sustain, release, signal);
        }
    };

    class EnvFxP {
        void* ffi;
        EnvFxP(const EnvFxP&);
        EnvFxP& operator=(const EnvFxP&);
    public:
        EnvFxP() : ffi(culsynth_env_i16_new()) {}
        ~EnvFxP() { culsynth_env_i16_free(ffi); }
        int32_t process(
            uint32_t sample_rate,
            uint32_t samples,
            const uint8_t* gate,
            const uint16_t* attack,
            const uint16_t* decay,
            const uint16_t* sustain,
            const uint16_t* release,
            uint16_t* signal)
        {
            return culsynth_env_i16_process(ffi, sample_rate, samples, gate,
                attack, decay, sustain, release, signal);
        }
    };

    class Filt {
        void* ffi;
        Filt(const Filt&);
        Filt& operator=(const Filt&);
    public:
        Filt() : ffi(culsynth_filt_f32_new()) {}
        ~Filt() { culsynth_filt_f32_free(ffi); }
        int32_t process(
            uint32_t sample_rate,
            uint32_t samples,
            const float* input,
            const float* cutoff,
            const float* resonance,
            float* low,
            float* band,
            float* high)
        {
            return culsynth_filt_f32_process(ffi, sample_rate, samples, input,
                cutoff, resonance, low, band, high);
        }
    };

    class FiltFxP {
        void* ffi;
        FiltFxP(const FiltFxP&);
        FiltFxP& operator=(const FiltFxP&);
    public:
        FiltFxP() : ffi(culsynth_filt_i16_new()) {}
        ~FiltFxP() { culsynth_filt_i16_free(ffi); }
        int32_t process(
            uint32_t sample_rate,
            uint32_t samples,
            const int16_t* input,
            const uint16_t* cutoff,
            const uint16_t* resonance,
            int16_t* low,
            int16_t* band,
            int16_t* high)
        {
            return culsynth_filt_i16_process(ffi, sample_rate, samples, input,
                cutoff, resonance, low, band, high);
        }
    };

    class Osc {
        void* ffi;
        Osc(const Osc&);
        Osc& operator=(const Osc&);
    public:
        Osc() : ffi(culsynth_osc_f32_new()) {}
        ~Osc() { culsynth_osc_f32_free(ffi); }
        int32_t process(
            uint32_t sample_rate,
            uint32_t samples,
            const float* note,
            const float* tune,
            const float* shape,
            float* sin,
            float* tri,
            float* sq,
            float* saw)
        {
            return culsynth_osc_f32_process(ffi, sample_rate, samples, note,
                tune, shape, sin, tri, sq, saw);
        }
    };

    class OscFxP {
        void* ffi;
        OscFxP(const OscFxP&);
        OscFxP& operator=(const OscFxP&);
    public:
        OscFxP() : ffi(culsynth_osc_i16_new()) {}
        ~OscFxP() { culsynth_osc_i16_free(ffi); }
        int32_t process(
            uint32_t sample_rate,
            uint32_t samples,
            const uint16_t* note,
            const int16_t* tune,
            const uint16_t* shape,
            int16_t* sin,
            int16_t* tri,
            int16_t* sq,
            int16_t* saw)
        {
            return culsynth_osc_i16_process(ffi, sample_rate, samples, note,
                tune, shape, sin, tri, sq, saw);
        }
    };
}
#endif
#endif