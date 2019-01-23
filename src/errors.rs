//! Errors related to proving and verifying proofs.

/// Represents an error in proof creation, verification, or parsing.
#[derive(Fail, Clone, Debug, Eq, PartialEq)]
pub enum VMError {
    /// This error occurs when an individual point operation failed.
    #[fail(display = "Point operation failed.")]
    PointOperationFailed,

    /// This error occurs when a point is not a valid compressed Ristretto point
    #[fail(display = "Point decoding failed.")]
    InvalidPoint,

    /// This error occurs when data is malformed
    #[fail(display = "Format in invalid")]
    FormatError,

    /// This error occurs when data is malformed
    #[fail(display = "Transaction version does not permit extension instructions.")]
    ExtensionsNotAllowed,

    /// This error occurs when an instruction requires a copyable type, but a linear type is encountered.
    #[fail(display = "Item is not a copyable type.")]
    TypeNotCopyable,

    /// This error occurs when an instruction requires a data type.
    #[fail(display = "Item is not a data string.")]
    TypeNotData,

    /// This error occurs when an instruction requires a contract type.
    #[fail(display = "Item is not a contract.")]
    TypeNotContract,

    /// This error occurs when an instruction requires a value type.
    #[fail(display = "Item is not a value.")]
    TypeNotValue,

    /// This error occurs when an instruction requires a value or a wide value.
    #[fail(display = "Item is not a wide value.")]
    TypeNotWideValue,

    /// This error occurs when VM does not have enough items on the stack
    #[fail(display = "Stack does not have enough items")]
    StackUnderflow,

    /// This error occurs when VM is left with some items on the stack
    #[fail(display = "Stack is not cleared by the program")]
    StackNotClean,

    /// This error occurs when VM's uniqueness flag remains false.
    #[fail(display = "Tx ID is not made unique via `input` or `nonce`")]
    NotUniqueTxid,
}
