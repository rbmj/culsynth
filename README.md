# Janus

Janus is currently a DSP library and [nih-plug](https://github.com/robbert-vdh/nih-plug)
based virtual instrument written in Rust.

## Goals

The eventual goal is to create a synthesizer that can be deployed both as a virtual
instrument and on embedded hardware, to allow users to prototype sound designs
with the expressiveness of physical knobs and hardware then seamlessly move those
patches into a virtual instrument inside a DAW for further use without requiring
sampling or reprogramming.

Hence: Janus, with two faces - physical and virtual.

## Licensing

This project is licensed under the MIT License.  However, due to the licensing
of the bindings any plugins built in VST3 format are licensed under the terms of
the GPLv3.

## Dependencies

For Rust dependencies, see `Cargo.toml`.

On Linux, the following packages (Debian names provided, but should be similar on other
distros) need to be installed:
 - `libx11-xcb-dev`
 - `libxcb-icccm4-dev`
 - `libxcb-dri2-0-dev`
