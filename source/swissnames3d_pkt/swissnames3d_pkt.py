import csv
from typing import List, Tuple

OBJECT_TYPES = [
    #"Gipfel",
    #"Alpiner Gipfel",
    "Hauptgipfel",
]

# Download and extract from https://www.swisstopo.admin.ch/de/landschaftsmodell-swissnames3d
SWISSNAMES_PATH = 'source/swissnames3d_pkt/swissNAMES3D_PKT.csv'

def get_peaks()->List[Tuple[int,int,int,str]]:
    with open(SWISSNAMES_PATH, mode='r', newline='') as csvfile:
        reader = csv.DictReader(csvfile, delimiter=";")
        data = [row for row in reader]
    return [(int(row["N"]),int(row["E"]),int(row["Z"]),row["NAME"]) for row in list(filter(lambda row: (row["OBJEKTART"] in OBJECT_TYPES)and row["STATUS"] == "offiziell", data))]
