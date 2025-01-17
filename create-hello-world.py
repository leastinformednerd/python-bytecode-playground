from types import CodeType

def hello_world():
    CODESTRING = bytes([0x74, 0, # LOAD_GLOBAL 0 (print)
                        0x64, 1, # LOAD_CONSTANT 1 ("Hello World")
                        0x83, 1, # CALL_FUNC 1 (print("Hello World"))
                        0x1, 0, # POP_TOP (arg ignored)
                        0x64, 0, # LOAD_CONSTANT 0 None
                        0x53, 0] # RETURN (arg ignored, return None)
                    )

    return CodeType(
        0, #argount
        0, #posonlyargcount
        0, #kwonlyargcount
        0, #nlocals
        2, #stacksize
        2, #flags
        CODESTRING, #codestring
        (None, "Hello World", ), #consts
        ("print",), #names
        tuple(), #varnames
        "hello-world.py", #filename
        "hello_world", #name
        0, #firstlineno
        bytes(), #linetable
        tuple(), #exceptiontable?
    )

if __name__ == "__main__":
    import marshal, inspect
    # print(inspect.signature(CodeType.__init__))
    # exit()
    with open("hello_world.pyc", "wb") as f:
        marshal.dump(
            hello_world(),
            f
        )
