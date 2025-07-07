import compile
import parse
import create_function

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

if __name__ == "__main__":
    create_pyc_from_program("program.pl", "compiled.pyc")
