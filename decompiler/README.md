# Decompiler
This crates contains the decompiler python bytecode playground / sandpit

Currently the decompiler technically "works", in that it can decompile simple function bodies, however it has a few things I'd like to clear up:

[ ] Support for function definitions
[ ] A python front end that can automatically grab the relavent \_\_code\_\_ sections
[ ] Remove extraneous `continue`s at the end of loop bodies
[ ] Understanding or operator priority and automatic correct parenthesis usage
[ ] Actual unit tests, as opposed to eyeballing the output
