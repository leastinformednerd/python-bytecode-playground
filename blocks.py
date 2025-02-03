from dataclasses import dataclass
from collections import namedtuple
from collections.abc import Callable

# arg: int | Variable
Instruction = namedtuple("Instruction", ["opcode", "arg"])

@dataclass
class Block:
    """A class for blocks of instructions.
    Differs slightly from the way it's defined in the pseudo-haskell due to personal preference"""
    instructions: list[Instruction]
    height: int
    depth: int

    def then(self, other: Block) -> Block:
        return Block(
            self.instructions + other.instructions,
            self.height + other.height,
            min(self.depth, self.height + other.depth)
        )

    @staticmethod
    def from_instr(instr: InstrInfo, arg: int) -> Block:
        return Block(
            [Instruction(instr.opcode, arg)],
            instr.get_height(arg),
            instr.get_depth(arg),
        )

    @staticmethod
    def construct_jump(jump_header: Block, success: Block, failure: Block, jump_past: bool = False) -> Block:
        if jump_past:
            failure = failure.then(Block.from_instr(instructions_info_d["JUMP_FORWARD"], 1+2*len(failure.instructions)))

        return Block(
            update_last_arg(jump_header.instructions, 1+2*len(failure.instructions))
                + failure.instructions + success.instructions,
            jump_header.height + min(failure.height, success.height),
            min(jump_header.depth,
                jump_header.height + failure.depth,
                jump_header.height + success.depth)
        )

    @staticmethod
    def early_ret() -> Block:
        '''A function that returns the value in co_consts[0]'''
        return Block.from_instr(instruction_info_d["LOAD_CONST"], 0).\
            then(Block.from_instr(instruction_info_d["RETURN_VALUE"], 0))
    
    def __add__(self, other: Block) -> Block:
        return self.then(other)

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
    return instr_l[:-1] + with_arg_as_list(instr_l[-1], delta)

def with_arg_as_list(instr: Instruction, arg: int):
    if arg <= 255:
        return [Instruction(instr.opcode, arg)]
    
    assert arg <= 0xffffffff, f"Expected an arg less than 4 bytes long, was {len(arg.to_bytes())} bytes long"
    
    l = [Instruction(instructioninfo["EXTENDED_ARG"].opcode, byte) for byte in arg.to_bytes(4)[:-1] if byte != 0]
    l.append(Instruction(instr.opcode, arg & 0xff))
    return l
        


@dataclass
class InstrInfo:
    name: str
    opcode: int
    height : int | Callable[[Int], int]
    depth: int | Callable[[Int], int]

    def get_height(self, arg):
        if isinstance(self.height, int):
            return self.height

        return self.height(arg)
    
    def get_depth(self, arg):
        if isinstance(self.depth, int):
            return self.depth

        return self.depth(arg)

instruction_info_d = {name: InstrInfo(name, index, height, depth) for index, (name, height, depth) in enumerate([
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
    ("POP_EXCEPT", -1, -1),
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
    ("COPY_FREE_VARS", lambda i: i, 0),
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
    ] + [(f"<{x}>", -999, -999) for x in range(116, 149)]
    + [("RESUME", 0, 0)] + [(f"<{x}>", -999, -99) for x in range(150, 237)]
    # There's some more after this point but I don't know what they do (undocumented) or they have
    # an opcode greater than one byte which I thought is unencodable so not sure why they're there.
    # All the ones that are in the docs or I can infer can be implemented with others so I think that's ok for now
)}

instruction_info_l = list(instruction_info_d.values())
