use super::Acci;

use super::IntervalQual;

pub trait Co5Order {
    type Output;
    fn co5_order(self) -> Self::Output;
}

pub trait FromCo5Order<T> {
    fn from_co5_order(t: T) -> Self;
}

pub trait Qual {
    fn qual(&self) -> IntervalQual;
}

pub trait AbsQual {
    fn abs_qual(&self) -> IntervalQual;
}

pub trait AcciByQual {
    fn acci_by_qual(&self, qual: IntervalQual) -> Option<Acci>;
}
