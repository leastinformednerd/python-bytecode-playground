from dataclasses import dataclass, field
from typing import Any, Self, Literal

class Operation:
    pass

@dataclass
class ASTNode:
    op: Operation
    children: [Self] = field(default_factory = list)

@dataclass
class Constant(Operation):
    '''Type to represent values stored in co_consts'''
    value: Any

bin_op_argcodes = {
    '+' : 0,
    '-' : 10,
    '*' : 5,
    '/' : 11,
    '//' : 2,
    '%' : 6,
    '@' : 4,
    '**' : 8,
    '<<' : 3,
    '>>' : 9,
    '&' : 1,
    '|' : 7,
    '^' : 12,
}

bin_op_argcodes |= {
    key+'=' : code + 13 for key, code in bin_op_argcodes.items()
}

@dataclass
class BinaryOp(Operation):
    '''Type to effectively wrap the BINARY_OP instruction'''
    op: Literal['+', '-', '*', '/', '//', '%', '@', '**', '<<', '>>', '&', '|', '^','+=', '-=', '*=', '/=', '//=', '%=', '@=', '**=', '<<=', '>>=', '&=', '|=', '^=',]

    def arg(self):
        return bin_op_argcodes[self.op]

comp_op_argcodes = {symbol : value << 5 for value, symbol in enumerate(('<', '<=', '==', '!=', '>', '>='))}

@dataclass
class CompOp(Operation):
    '''Type to wrap the COMPARE_OP instruction. Is missing some undocumented paramater in the fourth-lowest bit'''
    op: Literal['<', '<=', '==', '!=', '>', '>=']
    force_convert = bool | None

    def arg(self) -> int:
        return (comp_op_argcodes[self.op] + 16 * (self.force_convert if self.force_convert is not None else 0)) | 8

@dataclass
class IsOp(Operation):
    '''Type to wrap the IS_OP instruction'''
    inverted: bool

@dataclass
class InOp(Operation):
    '''Type to wrap the IN_OP instruction'''
    inverted: bool

class IfElse(Operation):
    '''Type to represent if ... else ...'''
    pass

class If(Operation):
    '''Type to represent if ... '''
    pass

class Sequence(Operation):
    '''Type to represent sequential execution of all children'''
    pass

@dataclass
class LoadName(Operation):
    '''Type to represent loading a name onto the stack'''
    name: str
    local: bool
    func: bool

@dataclass
class StoreName(Operation):
    '''Type to represent storing a value into a name'''
    name: str
    local: bool

class Call(Operation):
    '''Type to represent a function / method call'''
    pass

@dataclass
class Define(Operation):
    name: str
    arg_names: [str]

class Return(Operation):
    pass
