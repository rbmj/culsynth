import './encoder.js';
import * as wasm from './pkg/culsynth_plugin.js';

class SynthProcessor extends AudioWorkletProcessor {
    constructor(options) {
        super(options)
        this.port.onmessage = e => {
            if (typeof e.data === "number") {
                if (this._synth) {
                    this._synth.on_message_js(e.data);
                }
            } else if (e.data.type === 'loadWasm') {
                var mod = new WebAssembly.Module(e.data.data);
                wasm.initSync(mod);
                this._synth = new wasm.SynthWorklet(sampleRate, 1);
            }
        }
    }
    process(inputs, outputs, parameters) {
        if (this._synth) {
            this._synth.process(outputs[0][0])
        }
        return true;
    }
}

registerProcessor('synth-processor', SynthProcessor)