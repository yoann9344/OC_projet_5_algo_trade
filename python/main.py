import time

from .one import brute_force_decimal, data as DATA


# data = DATA[:10]

# brute_force_decimal()


def time_it(nb_items):
    data = DATA[:nb_items]
    brute_force_decimal.cache_clear()
    ti = time.perf_counter()
    brute_force_decimal()
    tf = time.perf_counter()
    print(tf - ti)


if __name__ == "__main__":
    for i in range(1, 1000, 20):
        time_it(i)
