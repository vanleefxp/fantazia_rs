pub trait Co5Order<T> {
    fn co5_order(self) -> T;
}

pub trait FromCo5<T> {
    fn from_co5(t: T) -> Self;
}

pub trait FromMod<T> {
    fn from_mod(value: T) -> Self;
}