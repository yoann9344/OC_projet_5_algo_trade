import csv


with open("dataset2_Python+P7.csv", newline="") as file:
    reader = csv.reader(file)
    data = [row for row in reader]


# print(data)
