use crate::Spice;

/// Equivalent to [From] but requires you to have the [Spice] struct.
pub trait SpiceFrom<T> {
    fn spice_from(_: T, _: Spice) -> Self;
}

/// Equivalent to [Into] but requires you to have the [Spice] struct.
///
/// Will be implemented automatically for anything that has [SpiceFrom].
pub trait SpiceInto<T> {
    fn spice_into(self, _: Spice) -> T;
}

impl<T, U> SpiceInto<U> for T
where
    U: SpiceFrom<T>,
{
    fn spice_into(self, spice: Spice) -> U {
        U::spice_from(self, spice)
    }
}
