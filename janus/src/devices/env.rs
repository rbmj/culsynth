use super::*;

type EnvParamFxP = fixedmath::U3F13;

#[derive(Eq, PartialEq)]
enum EnvState {
    Release,
    Attack,
    Decay
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
    pub fn create() -> Self {
        Self {
            state: EnvState::Release,
            outbuf: [ScalarFxP::ZERO; STATIC_BUFFER_SIZE],
            setpoint: Self::SIGNAL_MIN,
            last: Self::SIGNAL_MIN
        }
    }
    fn one_over(x: fixedmath::U19F13) -> (fixedmath::U1F15, u32) {
        let mut shift = x.leading_zeros();
        let mut x_shifted = fixedmath::U1F31::from_bits(x.to_bits()).unwrapped_shl(shift);
        if x_shifted >= fixedmath::U1F31::SQRT_2 {
            shift -= 1;
            x_shifted = x_shifted.unwrapped_shr(1);
        }
        let x_shifted_trunc = fixedmath::U1F15::from_num(x_shifted);
        let x2 = fixedmath::I3F29::from_num(x_shifted_trunc.wide_mul(x_shifted_trunc));
        let one_minus_x = fixedmath::I3F29::ONE - fixedmath::I3F29::from_num(x_shifted);
        let result = x2 + one_minus_x + one_minus_x.unwrapped_shl(1);
        (fixedmath::U1F15::from_num(result), 18 - shift)
    }
    pub fn process(&mut self,
        gate: &[SampleFxP],
        attack: &[EnvParamFxP],
        decay: &[EnvParamFxP],
        sustain: &[ScalarFxP],
        release: &[EnvParamFxP]
    ) -> &[ScalarFxP] {
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
            const SAMPLE_RATE : u16 = 44100;
            // This is equivalen to saying rise time = 4 time constants...
            let sr = fixedmath::U16F0::from_bits(SAMPLE_RATE >> 1);
            let k = rise.wide_mul(sr) + fixedmath::U19F13::ONE;
            let (gain, shift) = Self::one_over(k);
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
        Box::into_raw(Box::new(EnvFxP::create()))
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

            let out = p.as_mut().unwrap().process(g, a, d, s, r);
            *signal = out.as_ptr().cast();
            out.len() as i32
        }
    }
}