import csv
import sys
from time import perf_counter_ns as timer
from copy import deepcopy
from dataclasses import dataclass
from decimal import Decimal
from operator import attrgetter

from matplotlib import pyplot


@dataclass
class Row:
    name: str
    price: Decimal
    profit: Decimal
    benefits: Decimal


@dataclass
class State:
    balance: Decimal
    earnings: Decimal
    actions: list[int]


def improved_recursived(data: list[Row], balance: Decimal):
    def recursive(state: State) -> State:
        best_state = deepcopy(state)
        earnings_incresed = False

        for i, row in enumerate(data):
            if i in state.actions:
                continue
            elif state.balance < row.price:
                continue
            elif earnings_incresed:
                break
            else:
                new_state = State(
                    state.balance - row.price,
                    state.earnings + row.benefits,
                    state.actions + [i],
                )

                if new_state.earnings > best_state.earnings:
                    best_state = new_state

                recursive_state = recursive(new_state)

                if recursive_state.earnings > best_state.earnings:
                    earnings_incresed = True
                    best_state = recursive_state

        return best_state

    initial_state = State(balance, Decimal("0"), [])
    return recursive(initial_state)


def load_data(dataset_number: int):
    with open(f"dataset/dataset{dataset_number}_Python+P7.csv") as f:
        reader = csv.DictReader(f)
        data: list[Row] = []

        for row in reader:
            price = Decimal(row["price"])
            profit = Decimal(row["profit"])
            benefits = price * profit / Decimal("100")
            data.append(
                Row(
                    name=row["name"],
                    price=price,
                    profit=profit,
                    benefits=benefits,
                )
            )

    return data


def sort_data(data: list[Row]) -> list[Row]:
    return list(reversed(sorted(data, key=attrgetter("profit"))))


def clean_data(data: list[Row]) -> list[Row]:
    return list(filter(lambda r: r.price > 0 and r.profit > 0, data))


def curves(data):
    nb_actions = []
    durations = []

    for nb_action in range(0, 1001, 10):
        nb_action = max(2, min(len(data), nb_action))
        nb_actions.append(nb_action)

        reduced_data = data[0:nb_action]
        data_cleaned = sort_data(reduced_data)

        start = timer()

        improved_recursived(data_cleaned, Decimal("500"))

        end = timer()
        # ns to us
        duration = int((end - start) * 1000)
        durations.append(duration)

    print(durations)
    pyplot.plot(nb_actions, durations, linewidth=4)
    # pyplot.plot(
    #     x_axis,
    #     y_axis_complexity,
    #     label="O(2^n)",
    #     linewidth=8,
    #     alpha=0.5,
    #     linestyle="--",
    # )
    # pyplot.yscale("log")
    pyplot.xlabel("Nombre d'actions.")
    pyplot.ylabel("Temps d'exécution.")
    # pyplot.legend()
    pyplot.show()


if __name__ == "__main__":
    data = load_data(2)
    data = clean_data(data)

    # if "--curves" in sys.argv:
    if "--curves" in sys.argv:
        curves(data)
    else:

        start = timer()
        data = sort_data(data)
        end = timer()
        duration_sort = int((end - start) / 1000)

        start = timer()
        result = improved_recursived(data, Decimal("500"))
        end = timer()
        duration = int((end - start) / 1000)

        print(f"sort duration {duration_sort} us")
        print(result, f"in {duration} us")

        for index in result.actions:
            print(data[index].name)
