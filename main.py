from source.terrain_viz import generate_img,PLOT_PATH
from source.anki_export import create_deck
import csv

with open('data.csv', mode='r', newline='') as csvfile:
    reader = csv.DictReader(csvfile)
    data = [row for row in reader]

names = []
paths = []

for row in data:
    name = str(row["name"])
    coords = [row["c1"],row["c2"],row["c3"],row["c4"]]
    coords.sort()
    south = int(coords[0])
    north = int(coords[1])
    west = int(coords[2])
    east = int(coords[3])
    print(f"generating image for {name}")
    generate_img(south, north, west, east, name, step=100, offline=False, static=False, animated_extension='gif')
    names.append(name)
    paths.append(f"{PLOT_PATH}/{name}.png")

create_deck(names, paths, "Mountains")
