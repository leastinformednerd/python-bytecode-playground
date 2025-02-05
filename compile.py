from c_ast import ASTNode
import c_ast

from blocks import Block, Variable, Constant, instruction_info_d
import blocks

from types import CodeType

from create_function import create_code_object, create_code_string, create_module

def compile_if_else(condition: ASTNode, success_branch: ASTNode, failure_branch: ASTNode | None):
    f=None
    match condition:
        # TODO: Add case for if not condition. f = jump_if_true
        
        case ASTNode(op = c_ast.CompOp() as op):
            op.force_convert = True
            condition.op = op
            f = blocks.jump_if_false

        case ASTNode(op = c_ast.IsOp(inverted=inverted), children = [ASTNode(op = c_ast.Constant(value = None)), rhs]):
            condition = rhs
            f = blocks.jump_if_not_none if not inverted else blocks.jump_if_none

        case ASTNode(op = c_ast.IsOp(inverted=inverted), children = [lhs, ASTNode(op = c_ast.Constant(value = None))]):
            condition = lhs
            f = blocks.jump_if_not_none if not inverted else blocks.jump_if_none

        case _:
            f = blocks.jump_if_false

    jump_header = compile(condition)
    assert jump_header.height == 1, f"If statement conditions are expected to have height 1, found {jump_header.height}"
    return jump_header.then(
        f(
            compile(failure_branch) if failure_branch is not None else blocks.noop_block(),
            compile(success_branch)
        )
    )

def compile(node: ASTNode) -> Block:
    assert isinstance(node, ASTNode), f"tried to compile non-ast value node with type {type(node)}"

    match node:
        case ASTNode(op = c_ast.Constant(value = value)):
            return Block.from_instr(instruction_info_d["LOAD_CONST"], Constant(value))
        
        case ASTNode(op = c_ast.BinaryOp() as op, children = [lhs, rhs]):
            lhs = compile(lhs)
            assert lhs.depth >= 0, f"Inputs to binary operations must not touch the stack below, found {lhs.depth=}"
            assert lhs.height == 1, f"Inputs to binary operations must have a height of 1, found {lhs.height}"

            rhs = compile(rhs)
            assert rhs.depth >= 0, f"Inputs to binary operations must not touch the stack below, found {rhs.depth=}"
            assert rhs.height == 1, f"Inputs to binary operations must have a height of 1, found {rhs.height}"
            
            return lhs.then(rhs).then(Block.from_instr(instruction_info_d["BINARY_OP"], op.arg()))
        
        case ASTNode(op = c_ast.CompOp() as op, children = [lhs, rhs]):
            lhs = compile(lhs)
            assert lhs.depth >= 0, f"Inputs to compare operations must not touch the stack below, found {lhs.depth=}"
            assert lhs.height == 1, f"Inputs to compare operations must have a height of 1, found {lhs.height}"

            rhs = compile(rhs)
            assert rhs.depth >= 0, f"Inputs to compare operations must not touch the stack below, found {rhs.depth=}"
            assert rhs.height == 1, f"Inputs to compare operations must have a height of 1, found {rhs.height}"
            
            return lhs.then(rhs).then(Block.from_instr(instruction_info_d["COMPARE_OP"], op.arg()))
        
        case ASTNode(op = c_ast.IsOp() as op, children = [lhs, rhs]):
            lhs = compile(lhs)
            assert lhs.depth >= 0, f"Inputs to `is` operations must not touch the stack below, found {lhs.depth=}"
            assert lhs.height == 1, f"Inputs to `is` operations must have a height of 1, found {lhs.height}"

            rhs = compile(rhs)
            assert rhs.depth >= 0, f"Inputs to `is` operations must not touch the stack below, found {rhs.depth=}"
            assert rhs.height == 1, f"Inputs to `is` operations must have a height of 1, found {rhs.height}"
            
            return lhs.then(rhs).then(Block.from_instr(instruction_info_d["IS_OP"], 1*op.inverted))
            
        case ASTNode(op = c_ast.InOp() as op, children = [lhs, rhs]):
            lhs = compile(lhs)
            assert lhs.depth >= 0, f"Inputs to `in` operations must not touch the stack below, found {lhs.depth=}"
            assert lhs.height == 1, f"Inputs to `in` operations must have a height of 1, found {lhs.height}"

            rhs = compile(rhs)
            assert rhs.depth >= 0, f"Inputs to `in` operations must not touch the stack below, found {rhs.depth=}"
            assert rhs.height == 1, f"Inputs to `in` operations must have a height of 1, found {rhs.height}"
            
            return lhs.then(rhs).then(Block.from_instr(instruction_info_d["IN_OP"], 1*op.inverted))

        case ASTNode(op = c_ast.IfElse(), children = [condition, true_branch, false_branch]):
            return compile_if_else(condition, true_branch, false_branch)
            
        case ASTNode(op = c_ast.If(), children = [condition, true_branch]):
            return compile_if_else(condition, true_branch, None)

        case ASTNode(op = c_ast.Sequence(), children=children):
            block = blocks.noop_block()
            for child in children:
                block = block.then(compile(child))
            return block

        case ASTNode(op = c_ast.LoadName(name = name, local = local, func = func)):
            var = Variable(name, local, func)
            return Block.from_instr(
                instruction_info_d[
                    "LOAD_LOCAL" if local else "LOAD_GLOBAL"
                ], var
            )

        case ASTNode(op = c_ast.StoreName(name = name, local = local)):
            var = Variable(name, local)
            return Block.from_instr(
                instruction_info_d[
                    "STORE_LOCAL" if local else "STORE_GLOBAL"
                ], var
            )

        case ASTNode(op = c_ast.Call(), children = children):
            func = compile(children[0])
            assert func.height == 2, f"function expressions must have height 2, found {func.height}"

            b = blocks.noop_block()
            for arg_expr in children[1:]:
                arg_expr = compile(arg_expr)
                assert arg_expr.height > 0, f"function arguments must have height > 0, found {arg_expr.height}"
                b = b.then(arg_expr)
            
            return func.then(b).then(Block.from_instr(instruction_info_d["CALL"], len(children)-1))

        case ASTNode(op = c_ast.Return(), children = []):
            return compile(ASTNode(op = c_ast.Constant(value = None))).then(
                Block.from_instr(instruction_info_d["RETURN_VALUE"], 0)
            )

        case ASTNode(op = c_ast.Return(), children = [val]):
            return compile(val).then(
                Block.from_instr(instruction_info_d["RETURN_VALUE"], 0)
            )
        
        case _:
            raise ValueError(f"Node {node} is malformed")

def block_to_code_string(block: Block):
    return create_code_string((instr.opcode, instr.arg) for instr in block.instructions)

def resolve_names(body: Block, filename: str, name: str) -> CodeType:
    '''Resolve names from the block and compile it into a code object. Currently doesn't support function defs'''
    consts = []
    locals = []
    globals = []
    consts_d = {}
    locals_d = {}
    globals_d = {}
    for instr in body.instructions:
        if isinstance(instr.arg, Variable):
            if instr.arg.local:
                if instr.arg.name not in locals_d:
                    locals_d[instr.arg.name] = len(locals)
                    locals.append(instr.arg.name)
                if instr.arg.func:
                    instr.arg = (locals_d[instr.arg.name] << 1) + 1
                else:
                    instr.arg = locals_d[instr.arg.name]
            else:
                if instr.arg.name not in globals_d:
                    globals_d[instr.arg.name] = len(globals)
                    globals.append(instr.arg.name)
                if instr.arg.func:
                    instr.arg = (globals_d[instr.arg.name] << 1) + 1
                else:
                    instr.arg = globals_d[instr.arg.name]
        elif isinstance(instr.arg, Constant):
            if instr.arg.value not in consts_d:
                consts_d[instr.arg.value] = len(consts)
                consts.append(instr.arg.value)
            instr.arg = consts_d[instr.arg.value]

    return create_code_object(
        0,0,0,len(locals),
        body.height, # this isn't correct but I'm lazy
        2,
        block_to_code_string(body),
        tuple(consts),
        tuple(globals),
        tuple(locals),
        filename,
        name,
        name,
        0,
        bytes(),
        bytes()
    )

if __name__ == "__main__":
    condition = ASTNode(op = c_ast.CompOp("=="), children = [
        ASTNode(
            op = c_ast.Call(), children = [
                ASTNode(op = c_ast.LoadName("int", False, True)),
                ASTNode(op = c_ast.Call(), children = [
                    ASTNode(op = c_ast.LoadName("input", False, True))
                ])
            ]
        ),
        ASTNode(op = c_ast.Constant(0))
    ])

    true_branch = ASTNode(op = c_ast.Return(), children = [ASTNode(op = c_ast.Call(), children = [
        ASTNode(op = c_ast.LoadName("print", False, True)),
        ASTNode(op = c_ast.Constant("input == 0"))
    ])])
    
    false_branch = ASTNode(op = c_ast.Return(), children = [ASTNode(op = c_ast.Call(), children = [
        ASTNode(op = c_ast.LoadName("print", False, True)),
        ASTNode(op = c_ast.Constant("input != 0"))
    ])])
    
    sample = ASTNode(op = c_ast.IfElse(), children = [condition, true_branch, false_branch])

    compiled = compile(sample)
    obj = resolve_names(compiled, "test_ast.py", "cond_test")

    create_module("test_ast.pyc", obj, "test_ast.py")
