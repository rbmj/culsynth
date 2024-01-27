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
    int16_t* signal
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
#endif
#endif