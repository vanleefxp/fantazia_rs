use malachite_base::num::arithmetic::traits::{Mod as _, ModMul as _, DivMod as _};

use super::{traits::{Co5Order, FromCo5Order, PitchNotation as _}, OPitch, OStep};

pub(crate) const CO5_ORDER: [i8; 7] = [0, 2, 4, -1, 1, 3, 5];

impl Co5Order for OStep {
    type Output = i8;
    fn co5_order(self) -> Self::Output {
        CO5_ORDER[self as usize]
    }
}

impl FromCo5Order<i8> for OStep {
    fn from_co5_order(co5_order: i8) -> Self {
        (co5_order.mod_op(7) as u8)
            .mod_mul(4, 7)
            .try_into()
            .unwrap()
    }
}

impl Co5Order for OPitch {
    type Output = i8;
    fn co5_order(self) -> Self::Output {
        self.step.co5_order() + self.acci().0 * 7
    }
}

impl FromCo5Order<i8> for OPitch {
    fn from_co5_order(co5_order: i8) -> Self {
        let step = OStep::from_co5_order(co5_order);
        let tone = step.diatonic_tone() + (co5_order + 1).div_mod(7).0;
        OPitch { step, tone }
    }
}