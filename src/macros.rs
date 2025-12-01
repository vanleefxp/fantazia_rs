#[macro_export]
macro_rules! impl_add_by_conversion {
    ($($t1:ty, $t2:ty),*$(,)?) => {
        $(
            impl std::ops::Add<$t1> for $t1 {
                type Output = $t1;
                fn add(self, other: $t1) -> $t1 {
                    let converted_self: $t2 = self.into();
                    let converted_other: $t2 = other.into();
                    let sum_converted = converted_self + converted_other;
                    sum_converted.into()
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! impl_add_assign_by_add {
    ($($t:ty),*$(,)?) => {
        $(impl std::ops::AddAssign<$t> for $t {
            fn add_assign(&mut self, other: $t) {
                *self = *self + other;
            }
        })*
    };
}

#[macro_export]
macro_rules! impl_sub_assign_by_sub {
    ($($t:ty),*$(,)?) => {
        $(impl std::ops::SubAssign<$t> for $t {
            fn sub_assign(&mut self, other: $t) {
                *self = *self - other;
            }
        })*
    };
}

#[macro_export]
macro_rules! impl_sum_bisect {
    ($t:ty, $init:expr) => {
        impl std::iter::Sum for $t {
            fn sum<I>(xs: I) -> $t
            where
                I: Iterator<Item = $t>,
            {
                let mut stack = Vec::new();
                for (i, x) in xs.enumerate() {
                    let mut s = x;
                    for _ in 0..(i + 1).trailing_zeros() {
                        s += stack.pop().unwrap();
                    }
                    stack.push(s);
                }
                let mut s = $init;
                for x in stack.into_iter().rev() {
                    s += x;
                }
                s
            }
        }

        impl<'a> std::iter::Sum<&'a $t> for $t {
            fn sum<I>(xs: I) -> $t
            where
                I: Iterator<Item = &'a $t>,
            {
                let mut stack = Vec::new();
                for (i, x) in xs.enumerate() {
                    let mut s = x.clone();
                    for _ in 0..(i + 1).trailing_zeros() {
                        s += stack.pop().unwrap();
                    }
                    stack.push(s);
                }
                let mut s = $init;
                for x in stack.into_iter().rev() {
                    s += x;
                }
                s
            }
        }
    };
}

#[macro_export]
macro_rules! impl_from_mod {
    ($t:ty, $n: expr, $repr_num_t:ty; $($num_t:ty),+$(,)?) => {
        $(
            impl FromMod<$num_t> for $t {
                fn from_mod(value: $num_t) -> Self {
                    <$t>::try_from(value.mod_op($n) as $repr_num_t).unwrap()
                }
            }
        )*
    };
}