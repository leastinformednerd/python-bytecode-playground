from dataclasses import dataclass, field
from collections.abc import Callable
from typing import Any, Self
import opcode
import dis

@dataclass
class Variable:
    name: str
    local: bool
    func: bool

@dataclass
class Constant:
    value: Any
    
# arg: int | Variable | Constant
@dataclass
class Instruction:
    opcode: int
    arg: int | Variable | Constant

@dataclass
class Block:
    """A class for blocks of instructions.
    Differs slightly from the way it's defined in the pseudo-haskell due to personal preference"""
    instructions: list[Instruction]
    height: int
    depth: int
    max_height: int
    args: [str] = field(default_factory = list)
    cells: tuple[Variable, ...] = field(default_factory = tuple)

    def then(self, other: Self) -> Self:
        assert len(self.args) == 0, "tried to concatenate a function block"
        assert len(other.args) == 0, "tried to concatenate a function block"
        return Block(
            self.instructions + other.instructions,
            self.height + other.height,
            min(self.depth, self.height + other.depth),
            max(self.height + other.max_height, self.max_height, other.max_height),
        )

    @staticmethod
    def from_instr(instr: "InstrInfo", arg: int | Variable | Constant) -> Self:
        if isinstance(arg, int):
            return Block(
                [Instruction(instr.opcode, arg)] + [Instruction(0,0)] * get_cache_number(instr.opcode),
                instr.get_height(arg),
                instr.get_depth(arg),
                instr.get_height(arg)
            )
        else:
            # We know that we can only get here in situations where the instruction has a constant height / depth
            # or it's a load that supports function call pushes
            return Block(
                [Instruction(instr.opcode, arg)] + [Instruction(0,0)] * get_cache_number(instr.opcode),
                instr.height if isinstance(instr.height, int) else 1+arg.func,
                instr.depth,
                instr.height if isinstance(instr.height, int) else 1 +arg.func
            )

    @staticmethod
    def construct_jump(jump_header: Self, success: Self, failure: Self, jump_past: bool = False) -> Self:
        if jump_past:
            failure = failure.then(Block.from_instr(instruction_info_d["JUMP_FORWARD"], len(success.instructions)))

        assert failure.height == success.height, f"It's invalid to form an if with different return sizes, {success.height=} {failure.height=}"

        return Block(
            update_last_arg(jump_header.instructions, len(failure.instructions))
                + failure.instructions + success.instructions,
            jump_header.height + success.height,
            min(jump_header.depth,
                jump_header.height + failure.depth,
                jump_header.height + success.depth),
            jump_header.height + success.height,
        )

    @staticmethod
    def early_ret() -> Self:
        '''A function that creates a block equivalent to `return None`'''
        return Block.from_instr(instruction_info_d["LOAD_CONST"], Constant(None)).\
            then(Block.from_instr(instruction_info_d["RETURN_VALUE"], 0))

    def pop_extraneous(self: Self) -> Self:
        return Block(
            self.instructions + [Instruction(instruction_info_d["POP_TOP"].opcode, 0)] * self.height,
            0,
            self.depth + min(self.height, 0),
            self.max_height
        )

    def __add__(self, other: Self) -> Self:
        return self.then(other)

cache_numbers = {op : sum(cache_vals.values()) for op, cache_vals in dis._cache_format.items()}

def get_cache_number(op: str | int) -> int:
    if isinstance(op, str):
        return cache_numbers.get(op, 0)
    
    if isinstance(op, int):
        return get_cache_number(dis.opname[op])
    
    raise TypeError(f"Expected argument of type 'str' or 'int' got {type(op)}")

def noop_block() -> Block:
    return Block([], 0, 0, 0)

def jump_if_true(true: Block, false: Block, jump_past: bool = False) -> Block:
    return Block.construct_jump(
        Block.from_instr(instruction_info_d["POP_JUMP_IF_TRUE"], 0),
        true, false, jump_past
    )

def jump_if_false(false: Block, true: Block, jump_past: bool = False) -> Block:
    return Block.construct_jump(
        Block.from_instr(instruction_info_d["POP_JUMP_IF_FALSE"], 0),
        false, true, jump_past
    )

def jump_if_none(none: Block, not_none: Block, jump_past: bool = False) -> Block:
    return Block.construct_jump(
        Block.from_instr(instruction_info_d["POP_JUMP_IF_NONE"], 0),
        none, not_none, jump_past
    )

def jump_if_not_none(not_none: Block, none: Block, jump_past: bool = False) -> Block:
    return Block.construct_jump(
        Block.from_instr(instruction_info_d["POP_JUMP_IF_NOT_NONE"], 0),
        not_none, none, jump_past
    )

def update_last_arg(instr_l: [Instruction], delta: int):
    for index, instr in enumerate(reversed(instr_l)):
        if instr.opcode != 0:
            break

    return instr_l[:-(1+index)] + with_arg_as_list(instr, delta) \
        + [Instruction(0,0)] * get_cache_number(instr.opcode)

def with_arg_as_list(instr: Instruction, arg: int):
    if arg <= 255:
        return [Instruction(instr.opcode, arg)]
    
    assert arg <= 0xffffffff, f"Expected an arg less than 4 bytes long, was {len(arg.to_bytes())} bytes long"
    
    instrs = [Instruction(instruction_info_d["EXTENDED_ARG"].opcode, byte) for byte in arg.to_bytes(4)[:-1] if byte != 0]
    instrs.append(Instruction(instr.opcode, arg & 0xff))
    return instrs
        


@dataclass
class InstrInfo:
    name: str
    opcode: int
    height : int | Callable[[int], int]
    depth: int | Callable[[int], int]

    def get_height(self, arg):
        if isinstance(self.height, int):
            return self.height

        return self.height(arg)
    
    def get_depth(self, arg):
        if isinstance(self.depth, int):
            return self.depth

        return self.depth(arg)

instruction_info_d = {name: InstrInfo(name, index, _height if isinstance(_height, int) and abs(_height) < 10 else _height, depth) for index, (name, _height, depth) in enumerate([
    ("CACHE", 0, 0),
    ("BINARY_SLICE", -2, -3),
    ("BINARY_SUBSCR", -1, -2),
    ("<3>",-999, -999),
    ("CHECK_EG_MATCH", 0, -2),
    ("CHECK_EXC_MATCH", 0, -1),
    ("CLEANUP_THROW", -2, -3), # This one is weirdly conditional but we have to assume the worst
    ("DELETE_SUBSCR", -2, -2),
    ("END_ASYNC_FOR", -2, -2), # This might have a height of -1, I'm not clear how exceptions work
    ("END_FOR", -1, -1),
    ("END_SEND",-2, -1),
    ("EXIT_INIT_CHECK - Undocumented", -999, -999),
    ("FORMAT_SIMPLE", 0, -1),
    ("FORMAT_WITH_SPEC", -1, -2),
    ("GET_AITER", 0, -1),
    ("GET_ANEXT", 1, 0),
    ("GET_ITER", 0, -1),
    ("RESERVED", -999, -999),
    ("GET_LEN", 1, -1),
    ("GET_YIELD_FROM_ITER", 0, -1),
    ("INTERPRETER_EXIT", 99999, 0), # I don't think I need this one but it should be able to be called anywhere
    ("LOAD_BUILD_CLASS", 1, 0),
    ("LOAD_LOCALS", 1, 0),
    ("MAKE_FUNCTION", 0, -1),
    ("MATCH_KEYS", 1, -2),
    ("MATCH_MAPPING", 1, -1),
    ("MATCH_SEQUENCE", 1, -1),
    ("NOP", 0, 0),
    ("NOT_TAKEN", -999, -999), # Undocumented
    ("POP_EXCEPT", -1, -1),
    ("POP_ITER", -1, -1), # Undocumented
    ("POP_TOP", -1, -1),
    ("PUSH_EXC_INFO", 1, -1),
    ("PUSH_NULL", 1, 0),
    ("RETURN_GENERATOR", 1, 0), # Not sure how to classify this one, it clears the current frame
    ("RETURN_VALUE", 0, -1), # This implements, stackwise, STACK.append(STACK.pop()). It does some frame stuff
    ("SETUP_ANNOTATIONS", 0, 0), # Ignores the stack entirely
    ("STORE_SLICE", -3, -4),
    ("STORE_SUBSCR", -2, -3),
    ("TO_BOOL", 0, -1),
    ("UNARY_INVERT", 0, -1),
    ("UNARY_NEGATIVE", 0, -1),
    ("UNARY_NOT", 0, -1),
    ("WITH_EXCEPT_START", -3, -4), # Not confident on this one
    ("BINARY_OP", -1, -2),
    ("BUILD_LIST", lambda i: -1*i + 1, lambda i: -1*i),
    ("BUILD_MAP", lambda i: -2*i + 1, lambda i: -2*i),
    ("BUILD_SET", lambda i: -1*i + 1, lambda i: -1*i),
    ("BUILD_SLICE", lambda i: -1*i + 1, lambda i: -1*i),
    ("BUILD_STRING", lambda i: -1*i + 1, lambda i: -1*i),
    ("BUILD_TUPLE", lambda i: -1*i + 1, lambda i: -1*i),
    ("CALL", lambda i: -1*i - 1, lambda i: -1*i - 2),
    ("CALL_FUNCTION_EX", -1, -2), # Not confident on this one
    ("CALL_INTRINSIC_1", 0, -1),
    ("CALL_INTRINSIC_2", -1, -2),
    ("CALL_KW", lambda i: -1*i - 2, lambda i: -1*i - 3),
    ("COMPARE_OP", -1, -2), # It actually doesn't specify but I assume it's binary
    ("CONTAINS_OP", -1, -2),
    ("CONVERT_VALUE", 0, -1),
    ("COPY", 1, lambda i: -1*i),
    ("COPY_FREE_VARS", 0, 0),
    ("DELETE_ATTR", -1, -1),
    ("DELETE_DEREF", 0, 0),
    ("DELETE_FAST", 0, 0),
    ("DELETE_GLOBAL", 0, 0),
    ("DELETE_NAME", 0, 0),
    ("DICT_MERGE", -1, lambda i: -1*i),
    ("DICT_UPDATE", -1, lambda i: -1*i),
    ("EXTENDED_ARG", 0, 0),
    ("FOR_ITER", 1, -1),
    ("GET_AWAITABLE", 0, -1),
    ("IMPORT_FROM", -1, -2),
    ("IMPORT_NAME", 1, -1),
    ("IS_OP", -1, -2),
    ("JUMP_BACKWARD", 0, 0),
    ("JUMP_BACKWARD_NO_INTERRUPT", 0, 0),
    ("JUMP_FORWARD", 0, 0),
    ("LIST_APPEND", -1, lambda i: -1*i),
    ("LIST_EXTEND", -1, lambda i: -1*i),
    ("LOAD_ATTR", lambda i: 1 if i&1 == 1 else 0, -1),
    ("LOAD_COMMON_CONSTANT", 1, 0), # Undocumented but I assume it's something like this if it's like LOAD_CONST
    ("LOAD_CONST", 1, 1),
    ("LOAD_DEREF", 1, 1),
    ("LOAD_FAST", 1, 1),
    ("LOAD_FAST_AND_CLEAR", 1, 1),
    ("LOAD_FAST_CHECK", 1, 1),
    ("LOAD_FAST_LOAD_FAST", 2, 1),
    ("LOAD_FROM_DICT_OR_DEREF", 0, -1),
    ("LOAD_FROM_DICT_OR_GLOBALS", 0, -1),
    ("LOAD_GLOBAL", lambda i: 2 if i&1 == 1 else 1, 1),
    ("LOAD_NAME", 1, 0),
    ("LOAD_SMALL_INT", 1, 0),
    ("LOAD_SPECIAL", 1, 0), # Undocumented so idk
    ("LOAD_SUPER_ATTR", lambda i: 2 if i&1 == 1 else 1, -3),
    ("MAKE_CELL", 0, 0),
    ("MAP_ADD", -3, -3),
    ("MATCH_CLASS", -2, -3),
    ("POP_JUMP_IF_FALSE", -1, -1),
    ("POP_JUMP_IF_NONE", -1, -1),
    ("POP_JUMP_IF_NOT_NONE", -1, -1),
    ("POP_JUMP_IF_TRUE", -1, -1),
    ("RAISE_VARARGS", 0, lambda i: -1 if i == 0 else (-2 if i == 1 else -3)),
    ("RERAISE", lambda i: 0 if i == 0 else -1, lambda i: -1 if i == 0 else -2),
    ("SEND", 0, -2), # This is weird but it is actually guaranteed to have the same height / depth
    ("SET_ADD", -2, -2),
    ("SET_FUNCTION_ATTRIBUTE", -1, -2),
    ("SET_UPDATE", -2 , -2),
    ("STORE_ATTR", -2, -2),
    ("STORE_DEREF", -1, -1),
    ("STORE_FAST", -1, -1),
    ("STORE_FAST_LOAD_FAST", 0, -1),
    ("STORE_FAST_STORE_FAST", 0, -2),
    ("STORE_GLOBAL", -1, -1),
    ("STORE_NAME", -1, -1),
    ("SWAP", 0, lambda i: max(-1, i)),
    ("UNPACK_EX", -999, -999), # > 2 byte instr so I'm going to pretend it doesn't exist
    ("UNPACK_SEQUENCE", lambda i: i-1, -1),
    ("YIELD_VALUE", 0, -1), # I'm actually unclear but I think this is correct
    ] + [(f"<{x}>", -999, -999) for x in range(118, 149)]
    + [("RESUME", 0, 0)] + [(f"<{x}>", -999, -99) for x in range(150, 239)]
    # There's some more after this point but I don't know what they do (undocumented) or they have
    # an opcode greater than one byte which I thought is unencodable so not sure why they're there.
    # All the ones that are in the docs or I can infer can be implemented with others so I think that's ok for now
)}

# for this_name, actual_name in zip(instruction_info_d.keys(), dis.opname):
#     if this_name != actual_name:
#         print (this_name, actual_name)

instruction_info_l = list(instruction_info_d.values())
