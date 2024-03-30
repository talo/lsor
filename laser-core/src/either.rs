use crate::driver::PushPrql;

pub fn if_then_else<Then, Else>(
    cond: bool,
    then: impl FnOnce() -> Then,
    r#else: impl FnOnce() -> Else,
) -> Either<Then, Else> {
    if cond {
        Either::Left(then())
    } else {
        Either::Right(r#else())
    }
}

pub enum Either<Left, Right> {
    Left(Left),
    Right(Right),
}

impl<Left, Right> PushPrql for Either<Left, Right>
where
    Left: PushPrql,
    Right: PushPrql,
{
    fn push_to_driver(&self, driver: &mut crate::driver::Driver) {
        match self {
            Either::Left(left) => left.push_to_driver(driver),
            Either::Right(right) => right.push_to_driver(driver),
        }
    }
}
