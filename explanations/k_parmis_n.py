import sys
from matplotlib import pyplot
from functools import lru_cache

sys.setrecursionlimit(10000)
pyplot.rcParams.update({"font.size": 22})


@lru_cache(maxsize=None)
def facto(n):
    if n == 1 or n == 0:
        return 1
    elif n < 0:
        raise ValueError("Seulement des entiers positifs !")
    return n * facto(n - 1)


def combinaisons_k_parmis_n(n, k):
    return int(facto(n) / (facto(k) * facto(n - k)))


def sum_k_parmis_n(n):
    return sum(combinaisons_k_parmis_n(n, k) for k in range(1, n + 1))


def show_result(method: callable, **kwargs):
    result = method(**kwargs)
    args = ", ".join(map(str, kwargs.values()))
    if result > 10000:
        print(f"{method.__name__}({args}) = {result:e}")
    else:
        print(f"{method.__name__}({args}) = {result}")


def courbe_logarithmique_sum_npn(end):
    x_axis = list(range(2, end))
    y_axis = [sum_k_parmis_n(x) for x in x_axis]
    y_axis_complexity = [2 ** x for x in x_axis]
    pyplot.plot(x_axis, y_axis, label="k parmis n", linewidth=4)
    pyplot.plot(
        x_axis,
        y_axis_complexity,
        label="O(2^n)",
        linewidth=8,
        alpha=0.5,
        linestyle="--",
    )
    pyplot.yscale("log")
    pyplot.xlabel("Nombre d'actions.")
    pyplot.ylabel("Nombre de combinaisons.")
    pyplot.legend()
    pyplot.show()


# show_result(combinaisons_k_parmis_n, n=4, k=2)
# show_result(sum_k_parmis_n, n=4)
# show_result(sum_k_parmis_n, n=999)
# show_result(sum_k_parmis_n, n=1000)

# Évolution logarithmique en fonction de n
courbe_logarithmique_sum_npn(end=800)
