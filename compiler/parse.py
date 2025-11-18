import lark
import c_ast

Tree = lark.Tree
Token = lark.Token

la = None
with open("grammar") as f:
    la = lark.Lark(f.read())

if la is None:
    exit()

op_mapping = {
    "lt" : "<",
    "gt" : ">",
    "eq" : "==",
    "sum" : "+",
    "sub" : "-",
    "mul" : "*",
    "div" : "/",
    "or" : "||"
}

def _simplify_tree(lark_tree):
    match lark_tree:
        case Tree(data = Token(value = 'atom' | 'atomic_symbol'), children = [child]):
            return simplify_tree(child)
        case Tree(data = Token(value = 'start'), children = children):
            return [c for c in(simplify_tree(child) for child in children) if c is not None]
        case Tree(data = Token(value = 'op'), children = [child]):
            return simplify_tree(child)
        case Tree(data = Token(value = 'number'), children = children):
            return int("".join(children))
        case Tree(data = Token(value = 'atom' | 'atomic_symbol')):
            raise ValueError(lark_tree)
        # case Tree(data = Token(value = 's_expression' | 'list' ), children=children):
        #     return (simplify_tree(children[0]), [c for c in(simplify_tree(child) for child in children[1:]) if c is not None])
        case Tree(data = token, children = children):
            match len(children):
                case 0:
                    return simplify_tree(token)
                case 1:
                    return simplify_tree(children[0])
                case _:
                    child = simplify_tree(children[0])
                    if isinstance(child, tuple) and len(child) > 1 and child[1] == []:
                        print("Here!!!")
                        return (child[0], [c for c in(simplify_tree(child) for child in children[1:]) if c is not None])
                    return (child, [c for c in(simplify_tree(child) for child in children[1:]) if c is not None])
        case Tree(data = Token(value = "bin_op"), children=[child]):
            print(f"Found bin op, {child=}")
            return simplify_tree(child)
        case Token(type = "RULE", value = "empty"):
            return None
        case Token(type = _type, value=value) if "ANON" in _type:
            return value
        case Token(value = value):
            return simplify_tree(value)
        case _:
            # print(f"{lark_tree=}")
            return lark_tree

def simplify_tree(tree):
    return patch(_simplify_tree(tree))

def patch(tree):
    match tree:
        case [elem]:
            return patch(elem)
        case list():
            return [patch(elem) for elem in tree]
        case (None, *elems):
            return patch(elems)
        case tuple():
            return tuple(patch(elem) for elem in tree)
        case _:
            return tree
    

def parse_tree(source: str):
    return la.parse(source)
# Tree(Token('RULE', 'start'), [Tree(Token('RULE', 's_expression'), [Tree(Token('RULE', 'atom'), [Tree(Token('RULE', 'atomic_symbol'), [Token('__ANON_0', ':def')])]), Tree(Token('RULE', 'empty'), []), Tree(Token('RULE', 'list'), [Tree(Token('RULE', 's_expression'), [Tree(Token('RULE', 'atom'), [Tree(Token('RULE', 'atomic_symbol'), [Token('__ANON_0', 'main')])]), Tree(Token('RULE', 'empty'), []), Tree(Token('RULE', 'list'), [Tree(Token('RULE', 's_expression'), [Tree(Token('RULE', 'atom'), [Tree(Token('RULE', 'atomic_symbol'), [Token('__ANON_0', 'print')])]), Tree(Token('RULE', 'empty'), [])]), Tree(Token('RULE', 's_expression'), [Tree(Token('RULE', 'atom'), [Tree(Token('RULE', 'literal'), [Tree(Token('RULE', 'number'), [Token('__ANON_1', '5'), Token('__ANON_1', '1'), Token('__ANON_1', '2')])])])])])])])])])

def _gen_ast(simplified_tree):
    match simplified_tree:
        case (str() as func,):
            return _gen_ast((func, []))
        case ("def", (name, [args, body])):
            if isinstance(args, str):
                args = [args, []]
            return c_ast.ASTNode(op = c_ast.StoreName(name, False), children = [
                c_ast.ASTNode(
                    op = c_ast.MakeFn([args[0], *args[1]], name, ()),
                    children = [
                        c_ast.ASTNode(op = c_ast.Return(), children=[_gen_ast(body)])
                    ]
                )
            ])
        case ("ifelse", (cond, true, false)):
            return c_ast.ASTNode(
                op = c_ast.IfElse(),
                children = [_gen_ast(cond), _gen_ast(true), _gen_ast(false)]
            )
                
        case ((("sum" | "sub" | "mul" | "div" | "or") as op, [lhs, rhs])):
            # print(f"bin op {op=} {lhs=} {rhs=}")
            lhs = _gen_ast(lhs)
            rhs = _gen_ast(rhs)
            # print(f"bin op {op=} {lhs=} {rhs=}")

            return c_ast.ASTNode(op = c_ast.BinaryOp(op = op_mapping[op]), children = [lhs, rhs])
        case (lhs, [("lt" | "eq" | "gt") as op, rhs]):
            return c_ast.ASTNode(op = c_ast.CompOp(op = op_mapping[op]), children = [_gen_ast(lhs), _gen_ast(rhs)])

        case (str() as func, tuple() as arg):
            return c_ast.ASTNode(op=c_ast.Call(), children = [
                c_ast.ASTNode(op = c_ast.LoadName(func, False, True)),
                _gen_ast(arg)
            ])
        
        case (str() as func, list() as args):
            return c_ast.ASTNode(op = c_ast.Call(), children = [
                c_ast.ASTNode(op = c_ast.LoadName(func, False, True)),
                *(_gen_ast(arg) for arg in args)
            ])
        case (str() as func1, str() as func2):
            return c_ast.ASTNode(op = c_ast.Call(), children = [
                c_ast.ASTNode(op = c_ast.LoadName(func1, False, True)),
                c_ast.ASTNode(op = c_ast.Call(), children = [
                    c_ast.ASTNode(op = c_ast.LoadName(func2, False, True))
                ])
            ])
        case list():
            return c_ast.ASTNode(op = c_ast.Sequence(), children = [_gen_ast(t) for t in simplified_tree])
        case int():
            return c_ast.ASTNode(op = c_ast.Constant(simplified_tree))
        case str():
            return c_ast.ASTNode(op = c_ast.LoadName(simplified_tree, True, False))
        case _:
            raise ValueError(f"Default found {simplified_tree=}")

def gen_ast(tree):
    return _gen_ast(tree).then(
        c_ast.ASTNode(op = c_ast.Return(), children = [
            c_ast.ASTNode(op = c_ast.Constant(None))
        ])
    )
    
if __name__ == "__main__":
    from pprint import pp
    
    with open("program.pl") as f:
        program = f.read()
    tree= la.parse(program)
    # pp(tree)
    simple = simplify_tree(tree)
    pp(simple)
    ast = _gen_ast(simple)
    print(ast)
