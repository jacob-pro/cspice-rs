use crate::Spice;

pub trait SpiceFrom<T> {
    fn from(_: T, _: Spice) -> Self;
}

pub trait SpiceInto<T> {
    fn into(self, _: Spice) -> T;
}

impl<T, U> SpiceInto<U> for T
where
    U: SpiceFrom<T>,
{
    fn into(self, spice: Spice) -> U {
        U::from(self, spice)
    }
}
