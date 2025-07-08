#![allow(dead_code)]
#[derive(Debug)]
pub enum ParseError {
    OddByteCount,
}

#[derive(Debug, Clone, Copy)]
pub struct ParseInstr {
    pub kind: ParseInstrKind,
    pub arg: i16,
}

impl ParseInstr {
    fn new(opcode: u8, arg: i16) -> Self {
        ParseInstr {
            kind: opcode.into(),
            arg,
        }
    }
}

impl ParseInstr {
    pub fn jump(&self) -> Option<i16> {
        match self.kind {
            ParseInstrKind::ForIter
            | ParseInstrKind::PopJumpIfFalse
            | ParseInstrKind::PopJumpIfTrue
            | ParseInstrKind::PopJumpIfNone
            | ParseInstrKind::PopJumpIfNotNone
            | ParseInstrKind::JumpForward => Some(self.arg + 1),
            ParseInstrKind::JumpBackward => Some(-self.arg + 2),
            _ => None,
        }
    }

    pub fn is_cond_jump(&self) -> bool {
        match self.kind {
            ParseInstrKind::ForIter
            | ParseInstrKind::PopJumpIfFalse
            | ParseInstrKind::PopJumpIfTrue
            | ParseInstrKind::PopJumpIfNone
            | ParseInstrKind::PopJumpIfNotNone => true,
            _ => false,
        }
    }

    pub fn is_nop(&self) -> bool {
        match self.kind {
            ParseInstrKind::Cache | ParseInstrKind::NotTaken | ParseInstrKind::Nop => true,
            _ => false,
        }
    }

    pub fn is_terminal(&self) -> bool {
        match self.kind {
            ParseInstrKind::ForIter
            | ParseInstrKind::PopJumpIfFalse
            | ParseInstrKind::PopJumpIfTrue
            | ParseInstrKind::PopJumpIfNone
            | ParseInstrKind::PopJumpIfNotNone
            | ParseInstrKind::JumpForward
            | ParseInstrKind::JumpBackward
            | ParseInstrKind::ReturnValue => true,
            _ => false,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum ParseInstrKind {
    LoadConst = 81,
    LoadGlobal = 89,
    LoadSmallInt = 91,
    LoadFast = 83,
    StoreFast = 109,
    StoreGlobal = 112,
    EndFor = 9,
    GetIter = 16,
    ForIter = 69,
    PopIter = 30,
    JumpBackward = 74,
    JumpForward = 76,
    PopJumpIfFalse = 97,
    PopJumpIfNone = 98,
    PopJumpIfNotNone = 99,
    PopJumpIfTrue = 100,
    BinaryOp = 44,
    CompareOp = 56,
    MakeFunction = 23,
    Call = 51,
    Cache = 0,
    ToBool = 39,
    PopTop = 31,
    NotTaken = 28,
    ReturnValue = 35,
    Resume = 149,
    Nop = 27,
}

impl From<u8> for ParseInstrKind {
    fn from(value: u8) -> Self {
        use ParseInstrKind::*;
        let ret = match value {
            0 => Cache,
            9 => EndFor,
            16 => GetIter,
            23 => MakeFunction,
            27 => Nop,
            28 => NotTaken,
            30 => PopIter,
            31 => PopTop,
            35 => ReturnValue,
            39 => ToBool,
            44 => BinaryOp,
            51 => Call,
            56 => CompareOp,
            69 => ForIter,
            74 => JumpBackward,
            76 => JumpForward,
            81 => LoadConst,
            83 => LoadFast,
            89 => LoadGlobal,
            91 => LoadSmallInt,
            97 => PopJumpIfFalse,
            98 => PopJumpIfNone,
            99 => PopJumpIfNotNone,
            100 => PopJumpIfTrue,
            109 => StoreFast,
            112 => StoreGlobal,
            149 => Resume,
            _ => todo!(
                "Currently there isn't support for the instruction with opcode {}",
                value
            ),
        };

        debug_assert_eq!(ret as u8, value);

        ret
    }
}

pub fn parse<'a>(code: &[u8]) -> Result<Vec<ParseInstr>, ParseError> {
    assert!(
        code.len() % 2 == 0,
        "On the python versions supported, bytecode instructions are all 2 bytes, so bytecode length must be even, found {}",
        code.len()
    );
    // SAFETY: Since we know that that `code` has an even number of bytes, and
    // that an [u8][x..x+2], and (u8, i8) have the same size, alignment and bit
    // validity, this is safe.
    //
    // In fact this is probably more safe since it represents things semantically
    // equivalent to how they are seen by the interpreter
    let mut code: &[(u8, i8)] = {
        unsafe { std::slice::from_raw_parts(std::mem::transmute(code.as_ptr()), code.len() / 2) }
    };
    let mut acc = Vec::new();
    while !code.is_empty() {
        let (rest, instr) = match parse_next(code) {
            Ok(rest) => rest,
            Err(err) => return Err(err),
        };
        acc.push(instr);
        code = rest;
    }

    Ok(acc)
}

fn parse_next<'a, 'b>(code: &'a [(u8, i8)]) -> Result<(&'a [(u8, i8)], ParseInstr), ParseError> {
    let (opcode, arg, rest) = match code {
        [(68, arg_ext), (opcode, arg), rest @ ..] => {
            (*opcode, ((*arg_ext as i16) << 8) + (*arg as i16), rest)
        }
        [(opcode, arg), rest @ ..] => (*opcode, *arg as i16, rest),
        _ => unreachable!("parse_next should only be called with non-empty `code`"),
    };

    Ok((rest, ParseInstr::new(opcode, arg)))
}
