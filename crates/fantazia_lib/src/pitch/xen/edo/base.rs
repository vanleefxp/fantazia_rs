use std::cell::OnceCell;

use malachite_base::num::arithmetic::traits::Gcd;

// use super::super::super::edo12::traits::PitchNotation;

const REORDER_ARG: [usize; 7] = [3, 0, 4, 1, 5, 2, 6];

pub struct EDO {
    edo: u64,
    fifth_size: OnceCell<u64>,
    diatonic: OnceCell<[u64; 7]>,
}

impl EDO {
    pub fn new(mut n: u64) -> Self {
        let mut fifth_size = (n as f64 * 1.5f64.log2()).round() as u64;
        let gcd = n.gcd(fifth_size);
        n /= gcd;
        fifth_size /= gcd;
        EDO {
            edo: n,
            fifth_size: OnceCell::from(fifth_size),
            diatonic: OnceCell::new(),
        }
    }

    pub fn edo(&self) -> u64 {
        self.edo
    }

    pub fn fifth_size(&self) -> u64 {
        *self
            .fifth_size
            .get_or_init(|| (self.edo as f64 * 1.5f64.log2()).round() as u64)
    }

    pub fn sharpness(&self) -> u64 {
        self.fifth_size() * 7 - self.edo
    }

    pub fn diatonic(&self) -> &[u64; 7] {
        self.diatonic.get_or_init(|| {
            let mut result = [0u64; 7];
            result[REORDER_ARG[0]] = self.edo - self.fifth_size();
            for i in 0..6 {
                result[REORDER_ARG[i + 1]] =
                    i as u64 * self.fifth_size() - self.edo * (i as u64 / 2);
            }
            result
        })
    }
}
