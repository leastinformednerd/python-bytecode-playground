Something Haskell-like

For example:
```Haskell
module name where

main = print (list (range 5))
```
is equivalent to
```Python
print(list(range(5)))
```

or for a slightly more involved (contrived) example
```Haskell
module name where

fib n=(0 | 1) = n -- Not super sure on the syntax for binds case. Or comments for that matter
fib n = let
	prev1 = fib (n-1)
	prev2 = fib (n-2)
	in prev1 + prev2

main = print (fib 6)
```
is equivalent to
```Python
def fib(n):
	if n < 2:
		return n
	return fib(n-1) + fib(n-2)
```

in general a module is defined something like this (in pseudo-syntax for a format I half remember and whose name I've completely forgotten):
```
module := module_declaration imports definitions
module_declaration := "module" idenitifer "where"
imports := "import" (qual_import | unqual_import) (imports | None)
qual_import := identifier
unqual_import := identifiers "from" identifier
identifiers := identifier "," (identifiers | None)
definitions := definition (definitions | None)
definition := *explained later*
```

If one of the definitions is for a nilary function called "main" then it is executed as the entry point of the module. An equivalent of the if \_\_name\_\_ idiom would be something like
```Haskell
main = if __name__ == "__main__" then *whatever* else None
```

The meat of any module is the definitions it provides. They are defined as such
```
definition := (bound | function)
shared := "bound" identifier "=" expr
function := identifier (patterns | None) "=" expr
_expr := (literal | unary_op expr | expr binary_op expr | call | if_then | if_then_else | let_in)
expr := ( "(" _expr ")" | _expr )
exprs := expr (exprs | None)

literal := *literal parsing rules*
unary_op := *union of every unary operator*
binary_op := *union of every binary operator*
call := identifier exprs
if_then := "if" expr "then" expr
if_then_else := "if" expr "then" expr "else" expr
let_in = "let" definition "in" expr
```

This gives us everything we need except for patterns. They're roughly
```
patterns := pattern (patterns | None)
pattern = (identifier "@" | None) _pattern
_pattern = ( "(" raw_pattern ")" | raw_pattern )
raw_pattern = literal | identifier | identifier "(" patterns | None ")"
```
