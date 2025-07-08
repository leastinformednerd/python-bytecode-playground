#![allow(dead_code)]
use std::rc::Rc;

type Block = super::symbolic_evaluation::BlockToken;
pub type Name = Rc<str>;

#[derive(Debug, Clone)]
pub enum StackItem {
    Derived(Box<Instr>),
    Local(Name),
    Global(Name),
    Const(PyConst),
    Null,
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Plus,
    Minus,
    Times,
    Div,
    TrueDiv,
    Mod,
    MatMul,
    StarStar,
    ShiftLeft,
    ShiftRight,
    And,
    Or,
    Xor,

    PlusEquals,
    MinusEquals,
    TimesEquals,
    DivEquals,
    TrueDivEquals,
    ModEquals,
    MatMulEquals,
    StarStarEquals,
    ShiftLeftEquals,
    ShiftRightEquals,
    AndEquals,
    OrEquals,
    XorEquals,
}

impl BinaryOp {
    fn in_place(&self) -> bool {
        match self {
            Self::PlusEquals
            | Self::MinusEquals
            | Self::TimesEquals
            | Self::DivEquals
            | Self::TrueDivEquals
            | Self::ModEquals
            | Self::MatMulEquals
            | Self::StarStarEquals
            | Self::ShiftLeftEquals
            | Self::ShiftRightEquals
            | Self::AndEquals
            | Self::OrEquals
            | Self::XorEquals => true,
            _ => false,
        }
    }
}

impl TryFrom<u8> for BinaryOp {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use BinaryOp::*;
        Ok(match value {
            0 => Plus,
            13 => PlusEquals,
            10 => Minus,
            23 => MinusEquals,
            5 => Times,
            18 => TimesEquals,
            11 => Div,
            24 => DivEquals,
            2 => TrueDiv,
            15 => TrueDivEquals,
            6 => Mod,
            19 => ModEquals,
            4 => MatMul,
            17 => MatMulEquals,
            8 => StarStar,
            21 => StarStarEquals,
            3 => ShiftLeft,
            16 => ShiftLeftEquals,
            9 => ShiftRight,
            22 => ShiftRightEquals,
            1 => And,
            14 => AndEquals,
            7 => Or,
            20 => OrEquals,
            12 => Xor,
            25 => XorEquals,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct ComparisonOp {
    pub kind: ComparisonOpKind,
    pub force_convert: bool,
}

#[derive(Debug, Clone)]
pub enum ComparisonOpKind {
    LessThan,
    LessThanEquals,
    Equals,
    NotEqual,
    GreaterThan,
    GreaterThanEquals,
}

impl TryFrom<u8> for ComparisonOp {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let force_convert = value & 16 != 0;
        let kind = match value >> 5 {
            0 => ComparisonOpKind::LessThan,
            1 => ComparisonOpKind::LessThanEquals,
            2 => ComparisonOpKind::Equals,
            3 => ComparisonOpKind::NotEqual,
            4 => ComparisonOpKind::GreaterThan,
            5 => ComparisonOpKind::GreaterThanEquals,
            _ => return Err(()),
        };
        Ok(ComparisonOp {
            kind,
            force_convert,
        })
    }
}

pub type PyConst = Rc<PyConstInner>;

#[derive(Debug)]
pub enum PyConstInner {
    Int(i64),
    BigInt(String),
    CodeObject(CodeObject),
    None,
}

#[derive(Debug)]
pub struct CodeObject {
    // co_name
    pub name: Name,
    // co_qualname
    pub fqn: Name,
    // co_argcount
    pub arg_count: usize,
    // co_posonlyargcount
    pub pos_arg_count: usize,
    // co_kwonlyargcount
    pub kw_arg_count: usize,
    // co_nlocals, co_varnames
    pub locals: Vec<Name>,
    // co_cellvars
    pub cell_vars: Vec<Name>,
    // co_freevars
    pub free_vars: Vec<Name>,
    // co_code
    pub code: Vec<Instr>,
    // co_consts
    pub consts: Vec<PyConst>,
    // co_names
    pub globals: Vec<Name>,
    // co_filename
    pub filename: Name,
}

#[derive(Debug, Clone)]
/// `Instr`s are effectively expressions, expressed in terms of vm instructions
pub enum Instr {
    Cache,
    BinarySlice {
        container: StackItem,
        start: StackItem,
        end: StackItem,
    },
    BinarySubscr(StackItem, StackItem),
    CheckEgMatch(StackItem, StackItem),
    CheckExcMatch(StackItem),
    CleanupThrow(StackItem, StackItem, StackItem),
    DeleteSubscr(StackItem, StackItem),
    EndAsyncFor(StackItem, StackItem),
    EndFor(StackItem),
    EndSend(StackItem),
    ExitInitCheck(StackItem),
    FormatSimple(StackItem),
    FormatWithSpec(StackItem, StackItem),
    GetAiter(StackItem),
    GetAnext(StackItem),
    GetIter(StackItem),
    Reserved,
    GetLen(StackItem),
    GetYieldFromIter(StackItem),
    InterpreterExit,
    LoadBuildClass,
    LoadLocals,
    MakeFunction(StackItem),
    MatchKeys(StackItem, StackItem),
    MatchMapping(StackItem),
    MatchSequence(StackItem),
    Nop,
    NotTaken,
    PopExcept(StackItem),
    PopIter(StackItem),
    PopTop(StackItem),
    PushExcInfo(StackItem),
    PushNull,
    ReturnGenerator,
    ReturnValue(StackItem),
    SetupAnnotations,
    StoreSlice(StackItem, StackItem, StackItem, StackItem),
    StoreSubscr(StackItem, StackItem, StackItem),
    ToBool(StackItem),
    UnaryInvert(StackItem),
    UnaryNegative(StackItem),
    UnaryNot(StackItem),
    WithExceptStart(StackItem, StackItem, StackItem, StackItem),
    BinaryOp(BinaryOp, StackItem, StackItem),
    BuildList(Vec<StackItem>),
    BuildMap(Vec<StackItem>),
    BuildSet(Vec<StackItem>),
    BuildSlice(StackItem, StackItem, Option<StackItem>),
    BuildName(Vec<StackItem>),
    BuildTuple(Vec<StackItem>),
    Call {
        obj: StackItem,
        meth: StackItem,
        args: Vec<StackItem>,
    },
    CallFunctionEx(StackItem, StackItem),
    CallIntrinsic1(StackItem),
    CallIntrinsic2(StackItem, StackItem),
    CallKw {
        called: StackItem,
        pos_args: Vec<StackItem>,
        kw_args: Vec<StackItem>,
        names: Vec<Name>,
    },
    CompareOp(ComparisonOp, StackItem, StackItem),
    ContainsOp(StackItem, StackItem),
    ConvertValue(StackItem),
    Copy(StackItem),
    CopyFreeVars,
    DeleteAttr(StackItem),
    DeleteDeref,
    DeleteFast,
    DeleteGlobal,
    DeleteName,
    DictMerge {
        dict: StackItem,
        mapping: StackItem,
    },
    DictUpdate {
        dict: StackItem,
        mapping: StackItem,
    },
    ExtendedArg,
    ForIter {
        cond: StackItem,
        found_val: Block,
        exhausted: Block,
    },
    GetAwaitable(StackItem),
    ImportFrom(StackItem, StackItem),
    ImportName(StackItem),
    IsOp(StackItem, StackItem),
    JumpBackward(Block),
    JumpBackwardNoInterrupt(Block),
    JumpForward(Block),
    ListAppend {
        list: StackItem,
        item: StackItem,
    },
    ListExtend {
        list: StackItem,
        from: StackItem,
    },
    LoadAttr(StackItem, Name),
    LoadCommonConstant,
    LoadConst(PyConst),
    LoadDeref(Name),
    LoadFast(Name),
    LoadFastAndClear(Name),
    LoadFastCheck(Name),
    LoadFastLoadFast(Name, Name),
    LoadFromDictOrDeref(Name),
    LoadFromDictOrGlobals(Name),
    LoadGlobal(Name),
    LoadName(Name),
    LoadSmallInt(u8),
    LoadSpecial(Name),
    LoadSuperAttr(StackItem, StackItem, StackItem),
    MakeCell(Name),
    MapAdd(StackItem, StackItem, StackItem),
    MatchClass(StackItem, StackItem, StackItem),
    PopJumpIfFalse {
        cond: StackItem,
        met: Block,
        otherwise: Block,
    },
    PopJumpIfNone {
        cond: StackItem,
        met: Block,
        otherwise: Block,
    },
    PopJumpIfNotNone {
        cond: StackItem,
        met: Block,
        otherwise: Block,
    },
    PopJumpIfTrue {
        cond: StackItem,
        met: Block,
        otherwise: Block,
    },
    RaiseVarargs0,
    RaiseVarargs1,
    RaiseVarargs2,
    Reraise {
        ex: StackItem,
        last_instr: Option<usize>,
    },
    Send(StackItem, StackItem, Block),
    SetAdd(StackItem, StackItem),
    SetFunctionAttribute(StackItem, StackItem),
    SetUpdate(StackItem, StackItem),
    StoreAttr(StackItem, StackItem),
    StoreDeref(StackItem),
    StoreFast(Name, StackItem),
    StoreFastLoadFast(StackItem),
    StoreFastStoreFast(StackItem, StackItem),
    StoreGlobal(Name, StackItem),
    StoreName(StackItem),
    Swap(StackItem, StackItem),
    UnpackEx {
        seq: StackItem,
        before: u8,
        after: u8,
    },
    UnpackSequence(StackItem, u8),
    YieldValue(StackItem),
    Resume,
}
