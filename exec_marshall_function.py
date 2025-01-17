import marshal
with open("hello_world.pyc", "rb") as f:
   exec(marshal.load(f))
