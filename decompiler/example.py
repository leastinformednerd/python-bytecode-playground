def f(x):
    while x:
        if x * 2:
           print(2)
        else:
            print(3)

def g(x):
    for i in range(x):
        if i - 5 == 0:
            continue
        elif i - 7 == 8:
            break
    if i - 11 == 9:
        return 5

def h(x):
    for i in range(x):
        while x < 2:
            if x > 5:
                print(x)
            elif x < 3:
                print(x)
            else:
                print(x)
            print(x)
