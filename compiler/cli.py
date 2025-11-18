from argparse import ArgumentParser
from pathlib import PurePath
import create_function
import parse
import compile

argparser = ArgumentParser()
argparser.add_argument("-o", "--output")
argparser.add_argument("-d", "--dis", action="store_true")
argparser.add_argument("filename")

args = argparser.parse_args()

def create_pyc_from_program(in_path: str, out_path: str, dis_: bool = True):
    source = None
    with open(in_path) as f:
        source = f.read()
    ast = parse.gen_ast(parse.simplify_tree(parse.parse_tree(source)))
    code_object = compile.resolve_names(
        compile.compile(ast),
        in_path,
        "<module>"
    )

    if dis_:
        import dis
        dis.dis(code_object)
    
    create_function.create_module(out_path, code_object, out_path)

create_pyc_from_program(
    args.filename,
    args.output if args.output is not None else PurePath(args.filename).with_suffix(".pyc").as_posix(),
    args.dis
)
