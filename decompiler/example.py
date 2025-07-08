def f(x):
    while x:
        if x * 2:
           print(2)
        else:
            print(3)

print(list(f.__code__.co_code))
