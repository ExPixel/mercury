pub trait TypedHeader: Sized {
    type Error;

    fn parse(string: &str) -> Result<Self, Self::Error>;
}
