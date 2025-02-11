/// An enum representing a choice in modulation source
#[repr(u16)]
#[derive(Clone, Copy, Default)]
pub enum ModSrc {
    /// MIDI Note On velocity
    #[default]
    Velocity,
    /// MIDI Channel aftertouch
    Aftertouch,
    /// The modulation wheel (MIDI CC #1)
    ModWheel,
    /// Modulation envelope #1
    Env1,
    /// Modulation envelope #2
    Env2,
    /// LFO #1
    Lfo1,
    /// LFO #2
    Lfo2,
}

impl ModSrc {
    /// An array containing all possible `ModSrc` values, in order
    pub const ELEM: [ModSrc; Self::numel()] = [
        ModSrc::Velocity,
        ModSrc::Aftertouch,
        ModSrc::ModWheel,
        ModSrc::Env1,
        ModSrc::Env2,
        ModSrc::Lfo1,
        ModSrc::Lfo2,
    ];
    /// An iterator over all the different elements in `ModSrc`
    pub const fn elements() -> &'static [ModSrc] {
        &Self::ELEM
    }
    /// Convert a `u8` to a `ModSrc`
    pub const fn from_u8(val: u8) -> Option<Self> {
        if val as usize > Self::numel() {
            None
        } else {
            Some(Self::ELEM[val as usize])
        }
    }
    /// The first value in elements
    pub const fn min() -> Self {
        Self::Velocity
    }
    /// The last value in elements
    pub const fn max() -> Self {
        Self::Lfo2
    }
    /// The number of different modualtion sources
    pub const fn numel() -> usize {
        1 + Self::max() as usize - Self::min() as usize
    }
    /// The string representation of the modulation source
    pub const fn to_str(&self) -> &'static str {
        match self {
            Self::Velocity => "Velocity",
            Self::Aftertouch => "Aftertouch",
            Self::ModWheel => "Mod Wheel",
            Self::Env1 => "Envelope 1",
            Self::Env2 => "Envelope 2",
            Self::Lfo1 => "LFO 1",
            Self::Lfo2 => "LFO 2",
        }
    }
    /// Returns if this is a secondary modulation source.
    ///
    /// LFO2 and ENV2 are secondary sources, and cannot modulate themselves or
    /// each other.
    pub const fn is_secondary(&self) -> bool {
        match self {
            Self::Env2 => true,
            Self::Lfo2 => true,
            _ => false,
        }
    }
}

/// An enum representing a modulation destination
#[repr(u16)]
#[derive(Clone, Copy, PartialEq, Default)]
pub enum ModDest {
    /// The default is `ModDest::Null`, which is equivalent to no modulation
    #[default]
    Null,
    /// Course tune for oscillator 1, ranging from -32 to +32 semitones
    Osc1Course,
    /// Fine tune for oscillator 1, ranging from -2 to +2 semitones
    Osc1Fine,
    /// The wave shape (phase distortion) of oscillator 1
    Osc1Shape,
    /// The mix of the sine wave output for oscillator 1
    Osc1Sin,
    /// The mix of the square wave output for oscillator 1
    Osc1Sq,
    /// The mix of the triangel wave output for oscillator 1
    Osc1Tri,
    /// The mix of the sawtooth wave output for oscillator 1
    Osc1Saw,
    /// Course tune for oscillator 2, ranging from -32 to +32 semitones
    Osc2Course,
    /// Fine tune for oscillator 1, ranging from -2 to +2 semitones
    Osc2Fine,
    /// The wave shape (phase distortion) of oscillator 2
    Osc2Shape,
    /// The mix of the sine wave output for oscillator 2
    Osc2Sin,
    /// The mix of the square wave output for oscillator 2
    Osc2Sq,
    /// The mix of the triangle wave output for oscillator 2
    Osc2Tri,
    /// The mix of the sawtooth wave output for oscillator 2
    Osc2Saw,
    /// The mix of the dry signal from oscillator 1 in the output of the
    /// ring modulation section
    RingOsc1,
    /// The mix of the dry signal from oscillator 2 in the output of the
    /// ring modulation section
    RingOsc2,
    /// The mix of the wet (modulated) signal in the output of the ring
    /// modulation section
    RingMod,
    /// The filter cutoff frequency
    FiltCutoff,
    /// The filter resonance parameter
    FiltRes,
    /// The filter envelope modulation
    FiltEnv,
    /// The filter keyboard tracking
    FiltKbd,
    /// The filter velocity modulation
    FiltVel,
    /// The filter low-pass output mix
    FiltLow,
    /// The filter band-pass output mix
    FiltBand,
    /// The filter high-pass output mix
    FiltHigh,
    /// The filter envelope attack
    EnvFiltA,
    /// The filter envelope decay
    EnvFiltD,
    /// The filter envelope sustain
    EnvFiltS,
    /// The filter envelope release
    EnvFiltR,
    /// The VCA envelope attack
    EnvAmpA,
    /// The VCA envelope decay
    EnvAmpD,
    /// The VCA envelope sustain
    EnvAmpS,
    /// The VCA envelope release
    EnvAmpR,

    /// The rate/frequency of LFO 2, in Hz
    Lfo2Rate,
    /// The modulation depth of LFO 2, from 0 to 1
    Lfo2Depth,
    /// The attack of modulation envelope 2
    Env2A,
    /// The decay of modulation envelope 2
    Env2D,
    /// The sustain of modulation envelope 2
    Env2S,
    /// The release of modulation envelope 2
    Env2R,
}

impl ModDest {
    /// Env2/Lfo2 may not modulate themselves/each other, so call this function
    /// when evaluating their modulation matrices to remap these invalid routes
    /// to `Self::Null`
    pub const fn remove_secondary_invalid_dest(self) -> Self {
        match self {
            Self::Lfo2Rate => Self::Null,
            Self::Lfo2Depth => Self::Null,
            Self::Env2A => Self::Null,
            Self::Env2D => Self::Null,
            Self::Env2S => Self::Null,
            Self::Env2R => Self::Null,
            val => val,
        }
    }
    /// The string representation of this modulation destination.
    pub const fn to_str(&self) -> &'static str {
        match self {
            Self::Null => "NONE",
            Self::Osc1Course => "Osc1Course",
            Self::Osc1Fine => "Osc1Fine",
            Self::Osc1Shape => "Osc1Shape",
            Self::Osc1Sin => "Osc1Sin",
            Self::Osc1Sq => "Osc1Sq",
            Self::Osc1Tri => "Osc1Tri",
            Self::Osc1Saw => "Osc1Saw",
            Self::Osc2Course => "Osc2Course",
            Self::Osc2Fine => "Osc2Fine",
            Self::Osc2Shape => "Osc2Shape",
            Self::Osc2Sin => "Osc2Sin",
            Self::Osc2Sq => "Osc2Sq",
            Self::Osc2Tri => "Osc2Tri",
            Self::Osc2Saw => "Osc2Saw",
            Self::RingOsc1 => "RingOsc1",
            Self::RingOsc2 => "RingOsc2",
            Self::RingMod => "RingMod",
            Self::FiltCutoff => "FiltCutoff",
            Self::FiltRes => "FiltRes",
            Self::FiltEnv => "FiltEnv",
            Self::FiltKbd => "FiltKbd",
            Self::FiltVel => "FiltVel",
            Self::FiltLow => "FiltLow",
            Self::FiltBand => "FiltBand",
            Self::FiltHigh => "FiltHigh",
            Self::EnvFiltA => "EnvFiltA",
            Self::EnvFiltD => "EnvFiltD",
            Self::EnvFiltS => "EnvFiltS",
            Self::EnvFiltR => "EnvFiltR",
            Self::EnvAmpA => "EnvAmpA",
            Self::EnvAmpD => "EnvAmpD",
            Self::EnvAmpS => "EnvAmpS",
            Self::EnvAmpR => "EnvAmpR",
            Self::Lfo2Rate => "Lfo2Rate",
            Self::Lfo2Depth => "Lfo2Depth",
            Self::Env2A => "Env2A",
            Self::Env2D => "Env2D",
            Self::Env2S => "Env2S",
            Self::Env2R => "Env2R",
        }
    }
    /// The first modulation destination, in order
    pub const fn min() -> Self {
        Self::Null
    }
    /// The last modulation destination, in order
    pub const fn max() -> Self {
        Self::Env2R
    }
    /// The number of modulation destinations
    pub const fn numel() -> usize {
        Self::max() as usize + 1
    }
    /// The last modulation destination before the secondary destinations
    ///
    /// The secondary modulation destinations are invalid destinations from
    /// LFO2/ENV2 to avoid self/co-modulation
    pub const fn max_secondary() -> Self {
        Self::EnvAmpR
    }
    /// An iterator over all modulation destinations
    pub fn elements() -> impl core::iter::Iterator<Item = ModDest> {
        Self::elements_secondary_if(false)
    }
    /// An iterator over all non-secondary modulation destinations
    ///
    /// FIXME: Bad name
    pub fn elements_secondary() -> impl core::iter::Iterator<Item = ModDest> {
        Self::elements_secondary_if(true)
    }
    /// An iterator that excludes the secondary modulation destinations if the
    /// argument is true, and includes them if it is false
    pub fn elements_secondary_if(sec: bool) -> impl core::iter::Iterator<Item = ModDest> {
        let max = if sec {
            Self::max_secondary()
        } else {
            Self::max()
        };
        ((Self::min() as u16)..=(max as u16)).map(|x| unsafe { core::mem::transmute(x) })
    }
}

impl TryFrom<u16> for ModDest {
    type Error = &'static str;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value >= Self::min() as u16 && value <= Self::max() as u16 {
            unsafe { Ok(core::mem::transmute(value)) }
        } else {
            Err("ModDest out of bounds")
        }
    }
}

impl TryFrom<&str> for ModDest {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::elements()
            .find(|elem| value == elem.to_str())
            .ok_or("ModDest::try_from::<&str> parse failure")
    }
}

/// A struct to allow expressing the different modulation destinations for a
/// particular oscillator.  See [OSC1_MOD_DEST]/[OSC2_MOD_DEST] and
/// [Modulator]/[ModulatorFxP]
pub struct OscModDest {
    /// Course tune
    pub course: ModDest,
    /// Fine tune
    pub fine: ModDest,
    /// Wave shape
    pub shape: ModDest,
    /// Sine output
    pub sin: ModDest,
    /// Square output
    pub sq: ModDest,
    /// Triangle output
    pub tri: ModDest,
    /// Sawtooth output
    pub saw: ModDest,
}

/// The modulation destinations corresponding to oscillator 1
pub const OSC1_MOD_DEST: OscModDest = OscModDest {
    course: ModDest::Osc1Course,
    fine: ModDest::Osc1Fine,
    shape: ModDest::Osc1Shape,
    sin: ModDest::Osc1Sin,
    sq: ModDest::Osc1Sq,
    tri: ModDest::Osc1Tri,
    saw: ModDest::Osc1Saw,
};

/// The modulation destinations corresponding to oscillator 2
pub const OSC2_MOD_DEST: OscModDest = OscModDest {
    course: ModDest::Osc2Course,
    fine: ModDest::Osc2Fine,
    shape: ModDest::Osc2Shape,
    sin: ModDest::Osc2Sin,
    sq: ModDest::Osc2Sq,
    tri: ModDest::Osc2Tri,
    saw: ModDest::Osc2Saw,
};

/// A struct to allow expressing the different modulation destinations for a
/// particular oscillator.  See [ENV_AMP_MOD_DEST]/[ENV_FILT_MOD_DEST] and
/// [Modulator]/[ModulatorFxP]
pub struct EnvModDest {
    /// Envelope Attack
    pub attack: ModDest,
    /// Envelope Decay
    pub decay: ModDest,
    /// Envelope Sustain
    pub sustain: ModDest,
    /// Envelope Release
    pub release: ModDest,
}

/// The modulation destinations corresponding to the VCA envelope
pub const ENV_AMP_MOD_DEST: EnvModDest = EnvModDest {
    attack: ModDest::EnvAmpA,
    decay: ModDest::EnvAmpD,
    sustain: ModDest::EnvAmpS,
    release: ModDest::EnvAmpR,
};

/// The modulation destinations corresponding to the filter envelope
pub const ENV_FILT_MOD_DEST: EnvModDest = EnvModDest {
    attack: ModDest::EnvFiltA,
    decay: ModDest::EnvFiltD,
    sustain: ModDest::EnvFiltS,
    release: ModDest::EnvFiltR,
};
