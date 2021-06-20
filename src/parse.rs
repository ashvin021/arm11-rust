use nom::error::{ErrorKind, ParseError};
use nom::IResult;

pub struct ArmNomError<I> {
    pub kind: ArmNomErrorKind<I>,
    backtrace: Vec<ArmNomErrorKind<I>>,
}

pub enum ArmNomErrorKind<I> {
    Nom(I, ErrorKind),
    CondError,
    OpcodeError,
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

pub type NomResult<I, T> = IResult<I, T, ArmNomError<I>>;