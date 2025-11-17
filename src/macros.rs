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
