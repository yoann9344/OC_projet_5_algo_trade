import time
from functools import lru_cache
from operator import itemgetter

from one import improved_algo_recursive, get_data

NB_DATA_SET = 1
DATA, _, _ = get_data(NB_DATA_SET)
# DATA = 5 * DATA
data = None

# improved_algo_recursive()


@lru_cache(maxsize=100000)
# @lru_cache(maxsize=None)
def brute_force(actions=tuple(), benefits=0, balance=500, index=0):
    best_result = {
        "actions": tuple(),
        "benefits": 0,
        "balance": 500,
        "index": 0,
    }

    for i, row in enumerate(data[index:]):
        name = row["name"]
        price = row["d_price"]
        # profit = row["d_profit"]
        benef = row["d_benefits"]

        if balance < price:
            break

        params = {
            "actions": tuple(sorted(actions + (name,))),
            "benefits": benefits + benef,
            "balance": balance - price,
            "index": i,
        }
        result = brute_force(**params)

        if result["benefits"] > best_result["benefits"]:
            best_result = result

    return best_result


def change_function_scope(function, lru_cached=False, **kwargs):
    if lru_cached:
        function.cache_clear()
        f_globals = function.__wrapped__.__globals__
    else:
        f_globals = function.__globals__

    saved_values = f_globals.copy()

    # Add Object creation context
    f_globals.update(kwargs)

    try:
        result = function()
    finally:
        f_globals = saved_values

    return result


def time_it(nb_items, improved=False):
    data = DATA[:nb_items]
    if improved:
        method = improved_algo_recursive
        data.sort(key=itemgetter("profit"), reverse=True)
    else:
        method = brute_force
        data.sort(key=itemgetter("price"))

    ti = time.perf_counter()
    change_function_scope(method, lru_cached=True, data=data)
    tf = time.perf_counter()
    print(tf - ti)
    return tf - ti


def get_complexity(curve: dict[str, float]):
    import sys
    from math import log10 as log

    complexities = [
        "O(1)",
        "O(log n)",
        "O(n)",
        "O(n log n)",
        "O(n^2)",
        "O(n^2 log n)",
        "O(n^3)",
        "O(2^n)",
    ]

    perf = []
    sum_coef = dict.fromkeys(complexities, 0)
    for point in curve:
        num = point["nb_items"]
        t = int(point["duration"] * 1000000)
        if t <= 1 or num <= 1:
            print("continue", num, t)
            continue

        perf.append((num, t))
        n = num
        # debug(n, t)
        sum_coef["O(1)"] += t
        sum_coef["O(log n)"] += t / log(n)
        sum_coef["O(n)"] += t / n
        sum_coef["O(n log n)"] += t / (n * log(n))
        sum_coef["O(n^2)"] += t / n / n
        sum_coef["O(n^2 log n)"] += t / (n * n * log(n))
        sum_coef["O(n^3)"] += t / n / n / n
        sum_coef["O(2^n)"] += t / 2 ** n

    coef = {complexity: sum_ / len(perf) for complexity, sum_ in sum_coef.items()}
    # force coef for 2^n, cause coef for small n could be to totally different from hightest
    # and be cause the coef is the mean of all coef, the result is probably totally wrong
    # same for n^3
    coef["O(n^3)"] = (
        perf[-1][1] / perf[-1][0] ** 3 + perf[-2][1] / perf[-2][0] ** 3
    ) / 2
    coef["O(2^n)"] = (
        perf[-1][1] / 2 ** perf[-1][0] + perf[-2][1] / 2 ** perf[-2][0]
    ) / 2

    formula = dict.fromkeys(complexities, None)

    formula["O(1)"] = lambda n, complexity: coef[complexity]
    formula["O(log n)"] = lambda n, complexity: coef[complexity] * log(n)
    formula["O(n)"] = lambda n, complexity: coef[complexity] * n
    formula["O(n log n)"] = lambda n, complexity: coef[complexity] * n * log(n)
    formula["O(n^2)"] = lambda n, complexity: coef[complexity] * n * n
    formula["O(n^2 log n)"] = lambda n, complexity: coef[complexity] * n * n * log(n)
    formula["O(n^3)"] = lambda n, complexity: coef[complexity] * n * n * n
    formula["O(2^n)"] = lambda n, complexity: coef[complexity] * 2 ** n

    errors = dict.fromkeys(complexities, 0)

    def get_error(formula, complexity):
        try:
            sum_ = 0
            for n, t in perf:
                prediction = formula(n, complexity)
                sum_ += abs(prediction - t)
                # debug(n, t, prediction)
            return sum_ / len(perf)
        except OverflowError:
            print("overflow", complexity)
            return int(sys.maxsize)

    for complexity in complexities:
        errors[complexity] = get_error(formula[complexity], complexity)

    print(min(errors, key=errors.get))


if __name__ == "__main__":
    brute_force_improved = True
    curve = []
    for i in range(1, len(DATA), 1):
        duration = time_it(i, improved=brute_force_improved)
        curve.append({"nb_items": i, "duration": duration})
        if duration > 5:
            break

    print(get_complexity(curve))
    import csv

    if brute_force_improved:
        file_name = f"curve_complexity_dataset{NB_DATA_SET}.csv"
    else:
        file_name = f"curve_complexity_dataset{NB_DATA_SET}_brutal.csv"
    with open(file_name, "w+") as file:
        writer = csv.DictWriter(file, curve)

    import matplotlib.pyplot as plt
    import pandas as pd

    df = pd.DataFrame(curve)

    plt.figure(1)
    plt.plot(df["nb_items"], df["duration"])
    plt.show()
