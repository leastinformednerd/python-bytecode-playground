def f(x):
    while x:
        if x * 2:
           print(2)
           continue
        print(3)

def g(x):
    for i in range(x):
        if i - 5 == 0:
            continue
        if i - 7 == 8:
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

def i(a,b,c,d):
    print(a if b>c else d)

def j(x):
    y = int(x)
    acc = 0
    for i in range(y):
        acc += i
    if acc == (y*(y-1))//2:
        print("Correctly found that sum of 0..x = x*(x+1)/2")
    else:
        print("Incorrecly analysed the sum of 0..x")
