import dis
from collections.abc import Iterator
from importlib._bootstrap_external import _code_to_timestamp_pyc
from types import CodeType
import blocks

def create_op_call(op: str | int, arg: int | None) -> bytes:
    if isinstance(op, str):
        return create_op_call(dis.opmap[op], arg)

    if not isinstance(op, int):
        raise TypeError(f"Expected argument of type 'str' or 'int' got {type(op)}")

    return bytes([op, arg if arg is not None else 0])

def create_code_string(instrs: Iterator[(int | str, int)]):
    return b"".join(create_op_call(instr, arg) for instr,arg in instrs)

def create_code_object(
    argcount: int,
    posonlyargcount: int,
    kwonlyargcount: int,
    nlocals: int,
    stacksize: int,
    flags: int,
    codestring: bytes,
    consts: tuple,
    names: tuple,
    varnames: tuple,
    filename: str,
    name: str,
    qual_name: str,
    firstlineno: int,
    linetable: bytes,
    exceptiontable: bytes,
):
    '''I just want named arguments :('''
    return CodeType(
        argcount,
        posonlyargcount,
        kwonlyargcount,
        nlocals,
        stacksize,
        flags,
        codestring,
        consts,
        names,
        varnames,
        filename,
        name,
        qual_name,
        firstlineno,
        linetable,
        exceptiontable,
    )

def create_module(file: str, code: CodeType, source_name: str):
    main_code = create_code_string(
        [("RESUME", 0),
        ("LOAD_CONST", 0),
        ("MAKE_FUNCTION", 0),
        ("STORE_GLOBAL", 0),
        ("LOAD_GLOBAL", 1),
        ("CACHE",0),
        ("CACHE",0),
        ("CACHE",0),
        ("CACHE",0),
        ("CALL", 0),
        ("CACHE",0),
        ("CACHE",0),
        ("CACHE",0),
        ("POP_TOP", 0),
        ("LOAD_CONST", 1),
        ("RETURN_VALUE", 0)]
    )
    
    main = create_code_object(
        0,
        0,
        0,
        0,
        2,
        2,
        main_code,
        (code, None,),
        (code.co_name,),
        (),
        source_name,
        "<module>",
        "<module>",
        1,
        bytes(),
        bytes()
    )

    write_to_file(file, main)

def write_to_file(file, code):
    with open(file, "wb") as f:
        f.write(
            _code_to_timestamp_pyc(code)
        )
