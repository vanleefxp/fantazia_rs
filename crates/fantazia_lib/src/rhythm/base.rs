// use compact_str::CompactString;
// use tinyvec::TinyVec;

// type TimeSigVec = TinyVec<[SingleTimeSig; 3]>;
// type TimeSigNumVec = TinyVec<[u8; 3]>;

// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// pub enum TimeSigNum {
//     Simple(u8),
//     Additive(TimeSigNumVec),
// }

// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// pub struct SingleTimeSig {
//     pub(crate) num: TimeSigNum,
//     pub(crate) den: u8,
//     pub(crate) sym: Option<CompactString>,
// }

// impl SingleTimeSig {
//     pub fn new_simple(num: u8, den: u8, sym: Option<CompactString>) -> Self {
//         Self {
//             num: TimeSigNum::Simple(num),
//             den,
//             sym,
//         }
//     }

//     pub fn new_additive(num: impl Iterator<Item = u8>, den: u8, sym: Option<CompactString>) -> Self {
//         Self {
//             num: TimeSigNum::Additive(num.collect()),
//             den,
//             sym,
//         }
//     }

//     pub fn to_simple(&self) -> Self {
//         use TimeSigNum::*;
//         match &self.num {
//             Simple(_) => self.clone(),
//             Additive(nums) => {
//                 let num = nums.into_iter().sum();
//                 Self {
//                     num: TimeSigNum::Simple(num),
//                     den: self.den,
//                     sym: self.sym.clone(),
//                 }
//             }
//         }
//     }

//     pub fn into_simple(mut self) -> Self {
//         use TimeSigNum::*;
//         match self.num {
//             Simple(_) => self,
//             Additive(nums) => {
//                 let num = nums.into_iter().sum();
//                 self.num = TimeSigNum::Simple(num);
//                 self
//             }
//         }
//     }

//     pub fn into_num(self) -> TimeSigNum {
//         self.num
//     }

//     pub fn to_num(self) -> TimeSigNum {
//         self.num.clone()
//     }

//     pub fn num_ref(&self) -> &TimeSigNum {
//         &self.num
//     }

//     pub fn den(&self) -> u8 {
//         self.den
//     }

//     pub fn into_sym(self) -> Option<CompactString> {
//         self.sym
//     }

//     pub fn to_sym(self) -> Option<CompactString> {
//         self.sym.clone()
//     }

//     pub fn sym_ref(&self) -> Option<&CompactString> {
//         self.sym.as_ref()
//     }
// }

// impl Default for SingleTimeSig {
//     fn default() -> Self {
//         Self {
//             num: TimeSigNum::Simple(4),
//             den: 4,
//             sym: None,
//         }
//     }
// }

// pub struct InterchangeableTimeSig {
//     pub(crate) main: Box<TimeSig>,
//     pub(crate) alternative: Box<TimeSig>,
// }
// pub struct AggregateTimeSig(TimeSigVec);
// pub struct AlternatingTimeSig(TimeSigVec);

// pub struct OpenTimeSig {
//     sym: Option<CompactString>,
// }

// pub enum TimeSig {
//     Simple(SingleTimeSig),
//     Interchangeable(InterchangeableTimeSig),
//     Aggregate(AggregateTimeSig),
//     Alternating(AlternatingTimeSig),
//     Open(OpenTimeSig),
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BinaryDuration {
    pub(crate) kind: i8,
    pub(crate) dots: u8,
}

impl BinaryDuration {
    pub const fn new(kind: i8, dots: u8) -> Self {
        BinaryDuration { kind, dots }
    }

    pub const fn new_undotted(kind: i8) -> Self {
        BinaryDuration { kind, dots: 0 }
    }

    pub const fn kind(&self) -> i8 {
        self.kind
    }

    pub const fn dots(&self) -> u8 {
        self.dots
    }

    pub const fn undotted(&self) -> Self {
        BinaryDuration {
            kind: self.kind,
            dots: 0,
        }
    }
}
