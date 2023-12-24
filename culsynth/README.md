# CulSynth Library

This library contains all of the DSP logic used in the [CulSynth](https://github.com/rbmj/culsynth)
plugin.  See the main repository README for more information.

## Features

The library contains both fixed and floating point implementations of various
primitives used inside a synthesizer.  These include oscillators, a 2-pole
state variable filter, envelopes, and LFOs.  Additionally, it includes a basic
subtractive synth voice logic tying these primitives together and provides the
ability to modulate all of the parameters within the voice.

It is `#![no_std]` compatible by default and all memory used in th elibrary can
be entirely statically allocated.  This results in some design tradeoffs; see
the documentation for details.