def f(x):
    while x:
       print(2) 

print(list(f.__code__.co_code))
