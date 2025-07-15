# Decompiler
This crates contains the decompiler python bytecode playground / sandpit. It's based on symbolic evaluation of Python bytecode and some reasonably naieve matching on the shape of the control flow graph it forms

Currently the decompiler technically "works", in that it can decompile simple function bodies, however it has a few things I'd like to clear up:

- [ ] Support for function definitions

- [ ] A Python front end that can automatically grab the relavent \_\_code\_\_ sections (effectively required to make function support at all ergonomic)

- [ ] Remove extraneous `continue`s at the end of loop bodies

- [ ] Understanding or operator priority and automatic correct parenthesis usage

- [ ] Actual unit tests, as opposed to eyeballing the output

- [ ] Comprehension support, the lack of which prevents idiomatic Python code from being decompiled

## Examples

The body of f in
```python
def f(x):
  while x:
    if x * 2:
      print(2)
      continue
    print(3)
```
after being compiled by the CPython interpreter is decompied to
```python
while x:
  if x * 2:
    print(2)
    continue
  print(3)
  continue
```
with the single difference being in a superfluous continue at the end of the while loop's inner block, which would not effect the way it is recompiled (althogh I would like to address it in the future).

Somewhat notably the version of `f` where the `if .. continue` is replaced with an `if .. else` construct would decompile identically (the decompiler 'prefers' semantically identical 'early exits' from blocks over `else` blocks])

Another example that shows the limitations of the current implementation a bit more clearly is that the body of i in
```python
def i(a,b,c,d):
  print(a if b>c else d)
```

after compilation is decompiled to

```python
if (b > c):
  print(a)
  return None
print(d)
return None
```

which is semantically equivalent and would compile to the same code object, however is not very pretty, and is a very literal translation of the bytecode emitted by the compiler. This kind of construction is very common for the compiler to produce so it is a goal to make it nicer / easier to read
