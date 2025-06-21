//! Contains constant definitions for NRPNs

use super::modulation::{ModDest, ModSrc};
use wmidi::U7;

const NRPN_CATEGORY_CC: u8 = 0;
const NRPN_CATEGORY_MODULATION: u8 = 1;

pub enum NrpnMsb {
    Cc,
    Modulation(ModSrc),
}

impl NrpnMsb {
    pub fn from_msb(msb: U7) -> Option<Self> {
        let msb: u8 = msb.into();
        match msb >> 4 {
            NRPN_CATEGORY_CC => Some(Self::Cc),
            NRPN_CATEGORY_MODULATION => Some(NrpnMsb::Modulation(ModSrc::from_u8(msb & 0x0F)?)),
            _ => None,
        }
    }
    pub fn with_lsb(self, lsb: U7) -> Option<Nrpn> {
        match self {
            NrpnMsb::Cc => Some(Nrpn::Cc(wmidi::ControlFunction(lsb))),
            NrpnMsb::Modulation(src) => Some(Nrpn::Modulation(src, ModDest::from_u7(lsb)?)),
        }
    }
}

pub enum Nrpn {
    Cc(wmidi::ControlFunction),
    Modulation(ModSrc, ModDest),
}

impl Nrpn {
    pub fn from_bytes(msb: U7, lsb: U7) -> Option<Self> {
        NrpnMsb::from_msb(msb)?.with_lsb(lsb)
    }
    pub fn from_u14(value: wmidi::U14) -> Option<Self> {
        let value: u16 = value.into();
        let msb = (value >> 7) as u8;
        let lsb = (value & 0x7F) as u8;
        let msb = U7::from_u8_lossy(msb);
        let lsb = U7::from_u8_lossy(lsb);
        NrpnMsb::from_msb(msb)?.with_lsb(lsb)
    }
    pub fn to_bytes(&self) -> (U7, U7) {
        match self {
            Nrpn::Cc(cc) => (U7::from_u8_lossy(NRPN_CATEGORY_CC << 4), cc.0),
            Nrpn::Modulation(src, dest) => (
                U7::from_u8_lossy((NRPN_CATEGORY_MODULATION << 4) | *src as u8),
                U7::from_u8_lossy(*dest as u8),
            ),
        }
    }
    pub fn to_u14(&self) -> wmidi::U14 {
        match self {
            Nrpn::Cc(cc) => unsafe {
                let cc: u8 = cc.0.into();
                wmidi::U14::from_unchecked((NRPN_CATEGORY_CC as u16) << (4 + 7) | cc as u16)
            },
            Nrpn::Modulation(src, dest) => {
                let msb = (NRPN_CATEGORY_MODULATION << 4) | *src as u8;
                let lsb = *dest as u8;
                unsafe { wmidi::U14::from_unchecked(((msb as u16) << 7) | (lsb as u16)) }
            }
        }
    }
}
