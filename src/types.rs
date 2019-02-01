//! Core ZkVM stack types: data, variables, values, contracts etc.

use crate::transcript::TranscriptProtocol;
use bulletproofs::r1cs;
use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;

use crate::ops::Instruction;
use crate::txlog::UTXO;
use crate::errors::VMError;
use crate::predicate::Predicate;

#[derive(Debug)]
pub enum Item<'tx> {
    Data(Data<'tx>),
    Contract(Contract<'tx>),
    Value(Value),
    WideValue(WideValue),
    Variable(Variable),
    Expression(Expression),
    Constraint(Constraint),
}

#[derive(Debug)]
pub enum PortableItem<'tx> {
    Data(Data<'tx>),
    Value(Value),
}

#[derive(Debug)]
pub enum Data<'tx> {
    Opaque(&'tx [u8]),
    Witness(Box<DataWitness<'tx>>)
}

/// Prover's representation of the witness.
#[derive(Debug)]
pub enum DataWitness<'tx> {
    Program(Vec<Instruction<'tx>>),
    Predicate(PredicateWitness<'tx>), // maybe having Predicate and one more indirection would be cleaner - lets see how it plays out
    Commitment(CommitmentWitness),
    Scalar(Scalar),
    Input(Contract<'tx>, UTXO),
}

#[derive(Debug)]
pub struct Contract<'tx> {
    pub(crate) payload: Vec<PortableItem<'tx>>,
    pub(crate) predicate: Predicate,
}

#[derive(Debug)]
pub struct Value {
    pub(crate) qty: Variable,
    pub(crate) flv: Variable,
}

#[derive(Debug)]
pub struct WideValue {
    pub(crate) r1cs_qty: r1cs::Variable,
    pub(crate) r1cs_flv: r1cs::Variable,
    pub(crate) witness: Option<(Scalar, Scalar)>
}

#[derive(Copy, Clone, Debug)]
pub struct Variable {
    pub(crate) index: usize,
    // the witness is located indirectly in vm::VariableCommitment
}

#[derive(Clone, Debug)]
pub struct Expression {
    /// Terms of the expression
    pub(crate) terms: Vec<(r1cs::Variable, Scalar)>,
}

#[derive(Clone, Debug)]
pub enum Constraint {
    Eq(Expression,Expression),
    And(Vec<Constraint>),
    Or(Vec<Constraint>),
    // no witness needed as it's normally true/false and we derive it on the fly during processing.
    // this also allows us not to wrap this enum in a struct.
}

/// Prover's representation of the predicate tree with all the secrets
#[derive(Debug)]
pub enum PredicateWitness<'tx> {
    Key(Scalar),
    Program(Vec<Instruction<'tx>>),
    Or(Box<(PredicateWitness<'tx>, PredicateWitness<'tx>)>),
}

/// Prover's representation of the commitment secret: witness and blinding factor
#[derive(Debug)]
pub struct CommitmentWitness {
    value: Scalar,
    blinding: Scalar,
}

impl<'tx>  Item<'tx>{
 
    // Downcasts to Data type
    pub fn to_data(self) -> Result<Data<'tx>, VMError> {
        match self {
            Item::Data(x) => Ok(x),
            _ => Err(VMError::TypeNotData),
        }
    }

    // Downcasts to a portable type
    pub fn to_portable(self) -> Result<PortableItem<'tx>, VMError> {
        match self {
            Item::Data(x) => Ok(PortableItem::Data(x)),
            Item::Value(x) => Ok(PortableItem::Value(x)),
            _ => Err(VMError::TypeNotPortable),
        }
    }

    // Downcasts to Variable type
    pub fn to_variable(self) -> Result<Variable, VMError> {
        match self {
            Item::Variable(v) => Ok(v),
            _ => Err(VMError::TypeNotVariable),
        }
    }

    // Downcasts to Expression type (Variable is NOT casted to Expression)
    pub fn to_expression(self) -> Result<Expression, VMError> {
        match self {
            Item::Expression(expr) => Ok(expr),
            _ => Err(VMError::TypeNotExpression),
        }
    }

    // Downcasts to Value type
    pub fn to_value(self) -> Result<Value, VMError> {
        match self {
            Item::Value(v) => Ok(v),
            _ => Err(VMError::TypeNotValue),
        }
    }


    // Downcasts to WideValue type (Value is NOT casted to WideValue)
    pub fn to_wide_value(self) -> Result<WideValue, VMError> {
        match self {
            Item::WideValue(w) => Ok(w),
            _ => Err(VMError::TypeNotWideValue),
        }
    }

    // Downcasts to Contract type
    pub fn to_contract(self) -> Result<Contract<'tx>, VMError> {
        match self {
            Item::Contract(c) => Ok(c),
            _ => Err(VMError::TypeNotContract),
        }
    }
}

impl<'tx> Data<'tx> {
    /// Ensures the length of the data string
    pub fn ensure_length(self, len: usize) -> Result<Data<'tx>, VMError> {
        if self.bytes.len() != len {
            return Err(VMError::FormatError);
        }
        Ok(self)
    }

    /// Converts a bytestring to a 32-byte array
    pub fn to_u8x32(self) -> Result<[u8; 32], VMError> {
        let mut buf = [0u8; 32];
        buf.copy_from_slice(self.ensure_length(32)?.bytes);
        Ok(buf)
    }

    /// Converts a bytestring to a compressed point
    pub fn to_point(self) -> Result<CompressedRistretto, VMError> {
        Ok(CompressedRistretto(self.to_u8x32()?))
    }

    /// Converts a bytestring to a canonical scalar
    pub fn to_scalar(self) -> Result<Scalar, VMError> {
        Scalar::from_canonical_bytes(self.to_u8x32()?).ok_or(VMError::FormatError)
    }
}

impl Value {
    /// Computes a flavor as defined by the `issue` instruction from a predicate.
    pub fn issue_flavor(predicate: &Predicate) -> Scalar {
        let mut t = Transcript::new(b"ZkVM.issue");
        t.commit_bytes(b"predicate", predicate.0.as_bytes());
        t.challenge_scalar(b"flavor")
    }
}

// Upcasting all types to Item

impl<'tx> From<Data<'tx>> for Item<'tx> {
    fn from(x: Data<'tx>) -> Self {
        Item::Data(x)
    }
}

impl<'tx> From<Value> for Item<'tx> {
    fn from(x: Value) -> Self {
        Item::Value(x)
    }
}

impl<'tx> From<WideValue> for Item<'tx> {
    fn from(x: WideValue) -> Self {
        Item::WideValue(x)
    }
}

impl<'tx> From<Contract<'tx>> for Item<'tx> {
    fn from(x: Contract<'tx>) -> Self {
        Item::Contract(x)
    }
}

impl<'tx> From<Variable> for Item<'tx> {
    fn from(x: Variable) -> Self {
        Item::Variable(x)
    }
}

impl<'tx> From<Expression> for Item<'tx> {
    fn from(x: Expression) -> Self {
        Item::Expression(x)
    }
}

impl<'tx> From<Constraint> for Item<'tx> {
    fn from(x: Constraint) -> Self {
        Item::Constraint(x)
    }
}

// Upcast a portable item to any item
impl<'tx> From<PortableItem<'tx>> for Item<'tx> {
    fn from(portable: PortableItem<'tx>) -> Self {
        match portable {
            PortableItem::Data(x) => Item::Data(x),
            PortableItem::Value(x) => Item::Value(x),
        }
    }
}
