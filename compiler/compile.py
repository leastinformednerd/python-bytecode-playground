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
    failure_branch = compile(failure_branch) if failure_branch is not None else blocks.noop_block()
    assert jump_header.height == 1, f"If statement conditions are expected to have height 1, found {jump_header.height}"
    return jump_header.then(
        f(
            failure_branch,
            compile(success_branch) + Block.from_instr(instruction_info_d["JUMP_FORWARD"], len(failure_branch.instructions))
        )
    )

def compile(node: ASTNode) -> Block:
    assert isinstance(node, ASTNode), f"tried to compile non-ast value node with type {type(node)}"

    match node:
        case ASTNode(op = c_ast.Constant(value = int() as value)) if value <= 255:
            return Block.from_instr(instruction_info_d["LOAD_SMALL_INT"], value)

        case ASTNode(op = c_ast.Constant(value = value)):
            return Block.from_instr(instruction_info_d["LOAD_CONST"], Constant(value))
        
        case ASTNode(op = c_ast.BinaryOp() as op, children = [_lhs, _rhs]):
            try:
                lhs = compile(_lhs)
                assert lhs.depth >= 0, f"Inputs to binary operations must not touch the stack below, found {lhs.depth=}"
                assert lhs.height == 1, f"Inputs to binary operations must have a height of 1, found {lhs.height}"

                rhs = compile(_rhs)
                assert rhs.depth >= 0, f"Inputs to binary operations must not touch the stack below, found {rhs.depth=}"
                assert rhs.height == 1, f"Inputs to binary operations must have a height of 1, found {rhs.height}"
            except AssertionError as err:
                print(f"{_rhs=} {_lhs=}")
                raise err
            
            return lhs.then(rhs).then(Block.from_instr(instruction_info_d["BINARY_OP"], op.arg()))
        
        case ASTNode(op = c_ast.CompOp() as op, children = [lhs, rhs]):
            try:
                lhs = compile(lhs)
                assert lhs.depth >= 0, f"Inputs to compare operations must not touch the stack below, found {lhs.depth=}"
                assert lhs.height == 1, f"Inputs to compare operations must have a height of 1, found {lhs.height}"

                rhs = compile(rhs)
                assert rhs.depth >= 0, f"Inputs to compare operations must not touch the stack below, found {rhs.depth=}"
                assert rhs.height == 1, f"Inputs to compare operations must have a height of 1, found {rhs.height}"
            except AssertionError as err:
                print(f"{rhs=} {lhs=}")
                raise err
            
            return lhs.then(rhs).then(Block.from_instr(instruction_info_d["COMPARE_OP"], op.arg()))
        
        case ASTNode(op = c_ast.IsOp() as op, children = [lhs, rhs]):
            try:
                lhs = compile(lhs)
                assert lhs.depth >= 0, f"Inputs to `is` operations must not touch the stack below, found {lhs.depth=}"
                assert lhs.height == 1, f"Inputs to `is` operations must have a height of 1, found {lhs.height}"

                rhs = compile(rhs)
                assert rhs.depth >= 0, f"Inputs to `is` operations must not touch the stack below, found {rhs.depth=}"
                assert rhs.height == 1, f"Inputs to `is` operations must have a height of 1, found {rhs.height}"
            except AssertionError as err:
                print(f"{rhs=} {lhs=}")
                raise err
            
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
                    "LOAD_FAST" if local else "LOAD_GLOBAL"
                ], var
            )

        case ASTNode(op = c_ast.StoreName(name = name, local = local), children = [val]):
            var = Variable(name, local, False)
            return compile(val) + Block.from_instr(
                instruction_info_d[
                    "STORE_FAST" if local else "STORE_GLOBAL"
                ], var
            )

        case ASTNode(op = c_ast.Call(pop = pop), children = [func, *args]):
            func = compile(func)
            assert func.height == 2, f"function expressions must have height 2, found {func.height}"

            b = blocks.noop_block()
            for arg_expr in args:
                arg_expr = compile(arg_expr)
                assert arg_expr.height > 0, f"function arguments must have height > 0, found {arg_expr.height}"
                b = b.then(arg_expr)
            
            return func.then(b).then(Block.from_instr(instruction_info_d["CALL"], len(args)))\
                .then(Block.from_instr(instruction_info_d["POP_TOP"], 0) if pop else blocks.noop_block())

        case ASTNode(op = c_ast.Return(), children = []):
            return compile(ASTNode(op = c_ast.Constant(value = None))).then(
                Block.from_instr(instruction_info_d["RETURN_VALUE"], 0)
            )

        case ASTNode(op = c_ast.Return(), children = [val]):
            return compile(val).then(
                Block.from_instr(instruction_info_d["RETURN_VALUE"], 0)
            )

        case ASTNode(op = c_ast.MakeFn(args = args, name = name, closes_over = closes_over), children = [body]):
            f = Block.from_instr(instruction_info_d["RESUME"], 0) + compile(body)

            if len(closes_over) != 0:
                f = Block.from_instr(instruction_info_d["COPY_FREE_VARS"], len(closes_over)) + f

            assert f.height == 1 and f.depth >= 0, f"Functions must have height and depth >= got {f.height=} {f.depth=}"
            
            f.args = args
            f = resolve_names(f, "main.py", name)

            make_block = Block.from_instr(instruction_info_d["LOAD_CONST"], Constant(f)).then(
                Block.from_instr(instruction_info_d["MAKE_FUNCTION"], 0)
            )

            if len(closes_over) != 0:
                make_block = make_block.then(Block.from_instr(instruction_info_d["SET_FUNCTION_ATTRIBUTE"], 8)) # closure

            for var in closes_over:
                make_block = Block.from_instr(instruction_info_d["MAKE_CELL"], var)+ make_block
            make_block.cells = closes_over
            
            load_tuple  = blocks.noop_block()
            for var in closes_over:
                load_tuple = load_tuple + Block.from_instr(instruction_info_d["LOAD_FAST"], var)

            if len(closes_over) != 0:
                load_tuple = load_tuple + Block.from_instr(instruction_info_d["BUILD_TUPLE"], len(closes_over))

            return load_tuple + make_block

        case ASTNode(op = c_ast.DebugArbitaryBlock(block = block)):
            return block
        
        case _:
            raise ValueError(f"Node with op={node.op} len={len(node.children)} {node} is malformed")

def block_to_code_string(block: Block):
    return create_code_string((instr.opcode, instr.arg) for instr in block.instructions)

def resolve_names(body: Block, filename: str = "main.py", name: str = "main") -> CodeType:
    '''Resolve names from the block and compile it into a code object.'''
    #body.args = body.args # Why did I do this
    n_args = len(body.args)
    n_cells = len(body.cells)
    
    consts = []
    locals = body.args + list(body.cells)
    globals = []
    consts_d = {}
    locals_d = {name: index for index, name in enumerate(locals)}
    globals_d = {}
    for instr in body.instructions:
        if isinstance(instr.arg, Variable):
            if instr.arg.local:
                if instr.arg.name not in locals_d:
                    locals_d[instr.arg.name] = len(locals)
                    locals.append(instr.arg.name)
                if n_args <= len(locals)-1 < n_args + n_cells:
                    instr.opcode = instruction_info_d["LOAD_DEREF"]
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
        n_args,
        0,0,len(locals),
        2*body.max_height, # This should I think be body.max_height but I'm not calculating that properly rn, so
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
    # dbg_print = lambda x : ASTNode(op = c_ast.Call(), children = [
    #     ASTNode(op = c_ast.LoadName("print", False, True)),
    #     x
    # ]).then(ASTNode(c_ast.DebugArbitaryBlock(Block.from_instr(instruction_info_d["POP_TOP"], 0))))

    def dbg_print(*values):
        return ASTNode(op = c_ast.Call(pop = True), children = [
            ASTNode(op = c_ast.LoadName("print", False, True)),
        ] + list(values))

    g_body = ASTNode(op = c_ast.Return(), children = [
                ASTNode(op = c_ast.BinaryOp(op = "+"), children = [
                    ASTNode(op = c_ast.LoadName("x", True, False)),
                    ASTNode(op = c_ast.LoadName("y", True, False)),
                ])
            ])

    print(f"{compile(g_body)=}")
    # exit()
    
    f_body = ASTNode(op = c_ast.Return(), children = [
        ASTNode(op = c_ast.MakeFn(args = ["y"], name = "<anonymous>", closes_over = (Variable("x", True, False),)), children = [
            g_body
        ])
    ])

    # print(compile(f_body))
   
    define_f = ASTNode(op = c_ast.StoreName("f", False), children = [
        ASTNode(op = c_ast.MakeFn(args = ["x"], name = "f"), children = [f_body])
    ])

    call_f = ASTNode(op = c_ast.Call(), children = [
        ASTNode(op = c_ast.LoadName("f", False, True)),
        ASTNode(op = c_ast.Call(), children = [
            ASTNode(op = c_ast.LoadName("int", False, True)),
            ASTNode(op = c_ast.Call(), children = [
                ASTNode(op = c_ast.LoadName("input", False, True))
            ])
        ])
        # ASTNode(op = c_ast.Constant(5))
    ])

    define_f = compile(define_f)

    call_f = compile(dbg_print(call_f)).pop_extraneous()

    # print(f"{define_f.max_height=} {call_f.max_height=}")

    compiled = define_f + Block.early_ret()

    assert compiled.height >= 0 and compiled.depth >= 0, f"{compiled.height=} {compiled.depth=}"
    
    obj = resolve_names(compiled, "test_ast.py", "cond_test")

    import dis
    dis.dis(obj)
  
    create_module("test_ast.pyc", obj, "test_ast.py")
