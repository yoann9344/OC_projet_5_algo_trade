import csv
import os
import sys
import time
from copy import deepcopy
from decimal import Decimal, getcontext
from functools import lru_cache
from operator import itemgetter
from pprint import pprint


sys.setrecursionlimit(1000)
getcontext().prec = 10
DEBUG = os.environ.get("DEBUG", False)


def debug(*args, **kwargs):
    if DEBUG:
        print(*args, **kwargs)


def get_data(number=1):
    with open(f"../dataset{number}_Python+P7.csv") as file:
        reader = csv.reader(file)
        reader = csv.DictReader(file)
        data = []
        initial_data = []
        best_profit = 0
        d_best_profit = Decimal(0)
        count = 0
        for row in reader:
            initial_data.append(deepcopy(row))
            row["name"] = row["name"].replace("Share-", "")
            if float(row["profit"]) < 0 and float(row["price"]) < 0:
                continue
            # if float(row["profit"]) < 0:
            #     continue
            if float(row["profit"]) < 0 or float(row["price"]) < 0:
                continue
            prev_price = row["price"]
            prev_profit = row["profit"]
            row["d_price"] = abs(Decimal(prev_price))
            row["d_profit"] = abs(Decimal(prev_profit))
            # row["d_price"] = Decimal(prev_price)
            # row["d_profit"] = Decimal(prev_profit)

            row["price"] = abs(float(row["price"]))
            row["profit"] = max(0, float(row["profit"]))
            row["profit"] = abs(float(row["profit"]))
            if (
                row["price"] == 0
                # or str(row["price"]) != prev_price
                or row["profit"] == 0
                # or str(row["profit"]) != prev_profit
            ):
                # if row["price"] != 0:
                # debug("REMOVED", row, prev_price, prev_profit)
                continue
            if row["price"] < 10:
                count += 1
            if row["profit"] > best_profit:
                best_profit = row["profit"]
            if row["d_profit"].compare(d_best_profit) == 1:
                best_profit = row["d_profit"]
            row["benefits"] = row["price"] * row["profit"] / 100
            row["d_benefits"] = row["d_price"] * row["d_profit"] / Decimal(100)

            data.append(row)

    # print("Count :", count)
    # data.sort(key=itemgetter("benefits"), reverse=True)
    # data.sort(key=itemgetter("price"), reverse=True)
    # data.sort(key=itemgetter("price"))
    data.sort(key=itemgetter("profit"), reverse=True)
    return data, d_best_profit, initial_data


data, d_best_profit, initial_data = get_data()
# pprint(data)
print("D BEST PROFIT", d_best_profit)

# debug(data)
# pprint(data[:5])
# pprint(data[-5:])
# exit(0)

treated_names = set()


@lru_cache(maxsize=100000)
# @lru_cache(maxsize=None)
def brute_force(actions=tuple(), benefits=0, balance=500):
    # current_increase = Decimal(benefits)
    # current_increase = Decimal(0.0000)
    # initial_benefits = Decimal(0.0000)
    current_increase = 0.0000
    initial_benefits = 0.0000
    current_best = ""
    best_result = {
        "actions": actions,
        "benefits": benefits,
        "balance": balance,
    }
    debug(balance, actions)

    # if len(actions) > 1:
    #     return best_result

    for row in data:
        name = row["name"]
        price = row["price"]
        d_price = row["d_price"]
        d_profit = row["d_profit"]

        row_profit = row["profit"]
        row_benef = row["benefits"]

        # if Decimal(price).compare(Decimal(balance)) == 1:
        if price > balance:
            # if price < 1:
            #     print("BALANCE", price)
            continue
        elif price + 0.01 > balance:
            debug("FEW DIFF", price, balance)
            # continue
        # if (
        #     len(actions) > 0
        #     and (best_profit / Decimal(100) * Decimal(price)).compare(current_increase)
        #     == -1
        # ):
        if best_profit / 100 * price < current_increase:
            # print(
            #     "BREAK",
            #     (best_profit / Decimal(100) * Decimal(price)),
            #     current_best,
            #     initial_benefits,
            #     current_increase,
            #     ">",
            #     benefits,
            #     actions,
            # )
            break
        if name in actions:
            continue

        params = {
            # "actions": tuple(actions + (name,)),
            "actions": tuple(sorted(actions + (name,))),
            "benefits": benefits + row_benef,
            # "benefits": benefits + price * row_profit / 100,
            "balance": balance - price,
        }
        result = brute_force(**params)

        increase = result["benefits"] - best_result["benefits"]
        if result["benefits"] > best_result["benefits"]:
            current_increase += increase
            # current_increase = initial_benefits + Decimal(result["benefits"])
            best_result = result
            # current_best = name
    return best_result


@lru_cache(maxsize=100000)
# @lru_cache(maxsize=None)
def improved_algo_recursive(actions=tuple(), benefits=Decimal(0), balance=Decimal(500)):
    current_increase = Decimal(0)

    best_result = {
        "actions": actions,
        "benefits": benefits,
        "balance": balance,
    }
    debug(balance, actions)

    for row in data:
        name = row["name"]
        price = row["d_price"]
        row_benef = row["d_benefits"]

        if price.compare((balance)) == 1:
            # if price < 1:
            #     print("BALANCE", price)
            continue
        # elif price + Decimal(0.01) > balance:
        #     debug("FEW DIFF", price, balance)
        #     # continue

        # if best_profit / 100 * price < current_increase:
        # if (d_best_profit / Decimal(100) * price).compare(current_increase) == -1:
        #     # print("BEST inf", d_best_profit / Decimal(100) * price, current_increase)
        #     break
        if current_increase > 0:
            break
        if name in actions:
            continue

        params = {
            "actions": tuple(sorted(actions + (name,))),
            "benefits": benefits + row_benef,
            # "benefits": benefits + price * row_profit / Decimal(100),
            "balance": balance - price,
        }
        result = improved_algo_recursive(**params)

        increase = result["benefits"] - best_result["benefits"]
        # if result["benefits"] > best_result["benefits"]:
        if result["benefits"].compare(best_result["benefits"]) == 1:
            current_increase += increase
            best_result = result

    return best_result


def main():
    # best = brute_force()
    best = improved_algo_recursive()
    print(best)
    best_rows = list(
        filter(
            lambda d: d["name"].replace("Share-", "") in best["actions"],
            initial_data,
        )
    )
    # pprint(best_rows)
    balance_initial = sum(abs(Decimal(row["price"])) for row in best_rows)
    benefits_initial = sum(
        # abs(Decimal(row["price"])) * abs(Decimal(row["profit"])) / Decimal(100)
        Decimal(row["price"]) * Decimal(row["profit"]) / 100
        for row in best_rows
    )
    test = Decimal(0)
    for row in best_rows:
        print("best", row["profit"], row["price"])
        test += (
            abs(Decimal(row["price"], 5)) * abs(Decimal(row["profit"])) / Decimal(100)
        )
    print("Total cost (initial) :", balance_initial)
    print("Total benefits (initial) :", benefits_initial)
    print("Total benefits (initial absolute) :", test)

    print("Total cost :", 500 - best["balance"])
    print("Total benefits :", best["benefits"])


if __name__ == "__main__":
    # DEBUG = True
    ti = time.perf_counter()
    main()
    tf = time.perf_counter()
    print(tf - ti)
    import timeit

    print(
        "TIMEIT",
        timeit.timeit(improved_algo_recursive, globals=globals(), number=100),
    )
