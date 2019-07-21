from time import sleep

def runner(x):
    def innerMethod(y):
            print("x is", y)
            return y + 1

    while(x>0):
        x=innerMethod(x+1)
        sleep(0.01)


runner(1)