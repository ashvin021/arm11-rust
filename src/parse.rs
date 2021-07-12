use nom::error::{ContextError, ErrorKind, ParseError};
use nom::{ErrorConvert, IResult};

#[derive(Debug)]
pub struct ArmNomError<I> {
    pub kind: ArmNomErrorKind<I>,
    backtrace: Vec<ArmNomErrorKind<I>>,
}

#[derive(Debug, Copy, Clone)]
pub enum ArmNomErrorKind<I> {
    Nom(I, ErrorKind),
    Context(I, &'static str),
    Operand2Constant,
    HexadecimalValue,
    DecimalValue,
    SignedDecimalValue,
}

impl<I> ArmNomError<I> {
    pub fn new(kind: ArmNomErrorKind<I>) -> Self {
        ArmNomError {
            kind,
            backtrace: Vec::new(),
        }
    }
}

impl<I> ParseError<I> for ArmNomError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> ArmNomError<I> {
        ArmNomError::new(ArmNomErrorKind::Nom(input, kind))
    }

    fn append(input: I, kind: ErrorKind, mut other: Self) -> Self {
        other.backtrace.push(ArmNomErrorKind::Nom(input, kind));
        other
    }
}

impl<I> From<ArmNomError<I>> for nom::Err<ArmNomError<I>> {
    fn from(err: ArmNomError<I>) -> Self {
        nom::Err::Error(err)
    }
}

impl<I> ErrorConvert<ArmNomError<I>> for ArmNomError<(I, usize)> {
    fn convert(self) -> ArmNomError<I> {
        let mut new_backtrace = Vec::new();
        for k in self.backtrace {
            new_backtrace.push(k.convert());
        }
        ArmNomError {
            kind: self.kind.convert(),
            backtrace: new_backtrace,
        }
    }
}

impl<I> ContextError<I> for ArmNomError<I> {
    fn add_context(input: I, ctx: &'static str, mut other: Self) -> Self {
        other.backtrace.push(ArmNomErrorKind::Context(input, ctx));
        other
    }
}

impl<I> ErrorConvert<ArmNomErrorKind<I>> for ArmNomErrorKind<(I, usize)> {
    fn convert(self) -> ArmNomErrorKind<I> {
        match self {
            ArmNomErrorKind::Nom(t, k) => ArmNomErrorKind::Nom(t.0, k),
            ArmNomErrorKind::Context(t, c) => ArmNomErrorKind::Context(t.0, c),
            ArmNomErrorKind::Operand2Constant => ArmNomErrorKind::Operand2Constant,
            ArmNomErrorKind::HexadecimalValue => ArmNomErrorKind::HexadecimalValue,
            ArmNomErrorKind::DecimalValue => ArmNomErrorKind::DecimalValue,
            ArmNomErrorKind::SignedDecimalValue => ArmNomErrorKind::SignedDecimalValue,
        }
    }
}

pub type NomResult<I, T> = IResult<I, T, ArmNomError<I>>;
