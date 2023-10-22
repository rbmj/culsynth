use super::*;

use super::EnvParamFxP;

#[derive(Eq, PartialEq)]
enum EnvState {
    Release,
    Attack,
    Decay
}

pub struct EnvParams<'a, Smp> {
    pub attack: &'a [Smp],
    pub decay: &'a [Smp],
    pub sustain: &'a [Smp],
    pub release: &'a [Smp]
}


pub struct Env<Smp> {
    state : EnvState,
    outbuf : BufferT<Smp>,
    setpoint : Smp,
    last : Smp
}

impl<Smp: Float> Env<Smp> {
    // much silliness because generics...
    const GATE_THRESHOLD: Smp = Smp::ONE_HALF;
    const ATTACK_THRESHOLD: Smp = Smp::POINT_NINE_EIGHT;
    const SIGNAL_MAX: Smp = Smp::ONE;
    const SIGNAL_MIN: Smp = Smp::ZERO;
    pub fn new() -> Self {
        Self {
            state: EnvState::Release,
            outbuf: [Smp::ZERO; STATIC_BUFFER_SIZE],
            setpoint: Self::SIGNAL_MIN,
            last: Self::SIGNAL_MIN
        }
    }
    pub fn process(&mut self, gate: &[Smp], params: EnvParams<Smp>) -> &[Smp] {
        let attack = params.attack;
        let decay = params.decay;
        let sustain = params.sustain;
        let release = params.release;
        let numsamples = std::cmp::min(
            std::cmp::min(
                    std::cmp::min(attack.len(), decay.len()),
                    std::cmp::min(sustain.len(), release.len())),
            std::cmp::min(STATIC_BUFFER_SIZE, gate.len()));
        let setpoint_old = self.setpoint;
        for i in 0..numsamples {
            if gate[i] <= Self::GATE_THRESHOLD {
                self.state = EnvState::Release;
                self.setpoint = Self::SIGNAL_MIN;
            }
            else if self.state == EnvState::Release {
                self.state = EnvState::Attack;
                self.setpoint = Self::SIGNAL_MAX;
            }
            else if self.state == EnvState::Attack && self.last > Self::ATTACK_THRESHOLD {
                self.state = EnvState::Decay;
            }
            let rise = if self.state == EnvState::Attack {
                attack[i]
            }
            else if self.state == EnvState::Decay {
                self.setpoint = sustain[i];
                decay[i]
            }
            else {
                release[i]
            };
            // This is equivalen to saying rise time = 4 time constants...
            let sr_2 = SAMPLE_RATE >> 1;
            let k = rise * Smp::from(sr_2).unwrap() + Smp::ONE;
            let pro = setpoint_old + self.setpoint - self.last - self.last;
            let delta = pro / k;
            self.last = self.last + delta;
            self.outbuf[i] = self.last;
        }
        &self.outbuf[0..numsamples]
    }
}

pub struct EnvParamsFxP<'a> {
    pub attack: &'a [EnvParamFxP],
    pub decay: &'a [EnvParamFxP],
    pub sustain: &'a [ScalarFxP],
    pub release: &'a [EnvParamFxP]
}


pub struct EnvFxP {
    state : EnvState,
    outbuf : BufferT<ScalarFxP>,
    setpoint : fixedmath::I3F29,
    last : fixedmath::I3F29
}

impl EnvFxP {
    const GATE_THRESHOLD: SampleFxP = SampleFxP::lit("0.5");
    const ATTACK_THRESHOLD: ScalarFxP = ScalarFxP::lit("0.98");
    const SIGNAL_MAX: ScalarFxP = ScalarFxP::lit("0x0.FFFC");
    const SIGNAL_MIN: fixedmath::I3F29 = fixedmath::I3F29::lit("0x0.0004");
    pub fn new() -> Self {
        Self {
            state: EnvState::Release,
            outbuf: [ScalarFxP::ZERO; STATIC_BUFFER_SIZE],
            setpoint: Self::SIGNAL_MIN,
            last: Self::SIGNAL_MIN
        }
    }
    pub fn process(&mut self, gate: &[SampleFxP], params: EnvParamsFxP) -> &[ScalarFxP] {
        let attack = params.attack;
        let decay = params.decay;
        let sustain = params.sustain;
        let release = params.release;
        let numsamples = std::cmp::min(
            std::cmp::min(
                    std::cmp::min(attack.len(), decay.len()),
                    std::cmp::min(sustain.len(), release.len())),
            std::cmp::min(STATIC_BUFFER_SIZE, gate.len()));
        let setpoint_old = self.setpoint;
        for i in 0..numsamples {
            if gate[i] <= Self::GATE_THRESHOLD {
                self.state = EnvState::Release;
                self.setpoint = Self::SIGNAL_MIN;
            }
            else if self.state == EnvState::Release {
                self.state = EnvState::Attack;
                self.setpoint = fixedmath::I3F29::from_num(Self::SIGNAL_MAX);
            }
            else if self.state == EnvState::Attack && self.last > Self::ATTACK_THRESHOLD {
                self.state = EnvState::Decay;
            }
            let rise = if self.state == EnvState::Attack {
                attack[i]
            }
            else if self.state == EnvState::Decay {
                self.setpoint = std::cmp::max(fixedmath::I3F29::from_num(
                    std::cmp::min(sustain[i], Self::SIGNAL_MAX)),
                    Self::SIGNAL_MIN);
                decay[i]
            }
            else {
                release[i]
            };
            // This is equivalen to saying rise time = 4 time constants...
            let sr = fixedmath::U16F0::from_bits(SAMPLE_RATE >> 1);
            let k = rise.wide_mul(sr);
            let (gain, shift) = fixedmath::one_over_one_plus(k);
            let pro = fixedmath::I2F14::from_num(
                setpoint_old + self.setpoint - self.last.unwrapped_shl(1));
            let delta = pro.wide_mul_unsigned(gain).unwrapped_shr(shift);
            //println!("Env::Process setpoint = {} setpoint_old = {} last = {} delta = {}",
            //    self.setpoint, setpoint_old, self.last, delta);
            self.last += delta;
            self.outbuf[i] = ScalarFxP::from_num(self.last);
        }
        &self.outbuf[0..numsamples]
    }
}

mod bindings {
    use super::*;

    #[no_mangle]
    pub extern "C" fn janus_env_u16_new() -> *mut EnvFxP {
        Box::into_raw(Box::new(EnvFxP::new()))
    }

    #[no_mangle]
    pub extern "C" fn janus_env_u16_free(p: *mut EnvFxP) {
        if !p.is_null() {
            let _ = unsafe { Box::from_raw(p) };
        }
    }

    #[no_mangle]
    pub extern "C" fn janus_env_u16_process(
        p: *mut EnvFxP,
        samples: u32,
        gate: *const i16,
        attack: *const u16,
        decay: *const u16,
        sustain: *const u16,
        release: *const u16,
        signal: *mut *const u16,
        offset: u32
    ) -> i32 {
        if p.is_null()
            || gate.is_null()
            || attack.is_null()
            || decay.is_null()
            || sustain.is_null()
            || release.is_null()
            || signal.is_null()
        {
            return -1;
        }
        unsafe {
            let g = std::slice::from_raw_parts(
                gate.offset(offset as isize).cast::<SampleFxP>(), samples as usize);
            let a = std::slice::from_raw_parts(
                attack.offset(offset as isize).cast::<EnvParamFxP>(), samples as usize);
            let d = std::slice::from_raw_parts(
                decay.offset(offset as isize).cast::<EnvParamFxP>(), samples as usize);
            let s = std::slice::from_raw_parts(
                sustain.offset(offset as isize).cast::<ScalarFxP>(), samples as usize);
            let r = std::slice::from_raw_parts(
                release.offset(offset as isize).cast::<EnvParamFxP>(), samples as usize);
            let params = EnvParamsFxP { attack: a, decay: d, sustain: s, release: r };
            let out = p.as_mut().unwrap().process(g, params);
            *signal = out.as_ptr().cast();
            out.len() as i32
        }
    }

    
    #[no_mangle]
    pub extern "C" fn janus_env_f32_new() -> *mut Env<f32> {
        Box::into_raw(Box::new(Env::new()))
    }

    #[no_mangle]
    pub extern "C" fn janus_env_f32_free(p: *mut Env<f32>) {
        if !p.is_null() {
            let _ = unsafe { Box::from_raw(p) };
        }
    }

    #[no_mangle]
    pub extern "C" fn janus_env_f32_process(
        p: *mut Env<f32>,
        samples: u32,
        gate: *const f32,
        attack: *const f32,
        decay: *const f32,
        sustain: *const f32,
        release: *const f32,
        signal: *mut *const f32,
        offset: u32
    ) -> i32 {
        if p.is_null()
            || gate.is_null()
            || attack.is_null()
            || decay.is_null()
            || sustain.is_null()
            || release.is_null()
            || signal.is_null()
        {
            return -1;
        }
        unsafe {
            let g = std::slice::from_raw_parts(
                gate.offset(offset as isize), samples as usize);
            let a = std::slice::from_raw_parts(
                attack.offset(offset as isize), samples as usize);
            let d = std::slice::from_raw_parts(
                decay.offset(offset as isize), samples as usize);
            let s = std::slice::from_raw_parts(
                sustain.offset(offset as isize), samples as usize);
            let r = std::slice::from_raw_parts(
                release.offset(offset as isize), samples as usize);
            let params = EnvParams::<f32> { attack: a, decay: d, sustain: s, release: r };
            let out = p.as_mut().unwrap().process(g, params);
            *signal = out.as_ptr().cast();
            out.len() as i32
        }
    }
}