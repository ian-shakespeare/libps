#[derive(Clone, Debug, PartialEq)]
pub enum OperatorObject {
    // Stack
    Dup,
    Exch,
    Pop,
    Copy,
    Roll,
    Index,
    Clear,
    Count,
    Counttomark,
    Cleartomark,

    // Math
    Add,
    Div,
    Idiv,
    Mod,
    Mul,
    Sub,
    Abs,
    Neg,
    Ceiling,
    Floor,
    Round,
    Truncate,
    Sqrt,
    Atan,
    Cos,
    Sin,
    Exp,
    Ln,
    Log,
    Rand,
    Srand,
    Rrand,

    // Array
    Array,
    EndArray, // ]
    Length,
    Get,
    Put,
    Getinterval,
    Putinterval,
    Astore,
    Aload,
    Forall,
    Packedarray,
    Setpacking,
    Currentpacking,

    // Dict
    Dict,
    EndDict, // >>
    Maxlength,
    Begin,
    End,
    Def,
    Load,
    Store,
    Undef,
    Known,
    Where,
    Currentdict,
    Countdictstack,

    // Boolean
    Eq,

    // Type
    Type,

    // Errors
    Handleerror,
    Dictstackunderflow,
    Invalidaccess,
    Ioerror,
    Limitcheck,
    Rangecheck,
    Stackunderflow,
    Syntaxerror,
    Typecheck,
    Undefined,
    Undefinedresult,
    Unmatchedmark,
    Unregistered,
    Vmerror,
}
