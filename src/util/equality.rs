pub struct IsEq;

pub struct IsNotEq;

pub trait IsEqualityOp {}

impl IsEqualityOp for IsEq {}

impl IsEqualityOp for IsNotEq {}

trait CompEq<U, TypeEq: IsEqualityOp> {}

impl<T> CompEq<T, IsEq> for T {}

impl<T, U> CompEq<U, IsNotEq> for T {}

pub struct Final;

pub trait RecEqChecker<T, Mode, U1, U2, Eq1: IsEqualityOp> {
    fn eq_check();
}

impl<T> RecEqChecker<T, Final, (), (), IsEq> for () {
    fn eq_check() {}
}

impl<U1, U2, Eq1: IsEqualityOp, Eq2: IsEqualityOp, I1, I2, G, T>
    RecEqChecker<T, (G, I1, I2, Eq2), U1, U2, Eq1> for (U1, U2)
where
    U1: CompEq<T, Eq1>,
    U2: RecEqChecker<T, G, I1, I2, Eq2>,
{
    fn eq_check() {}
}

pub trait TupleLength {
    const LENGTH: usize;
}

impl<T1, T2> TupleLength for (T1, T2)
where
    T2: TupleLength,
{
    const LENGTH: usize = 1 + T2::LENGTH;
}
impl TupleLength for () {
    const LENGTH: usize = 0;
}
