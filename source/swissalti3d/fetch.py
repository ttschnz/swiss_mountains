import requests
import zipfile
import logging
import io
import csv

from typing import List, Tuple, Self

from . import cache
import utils
from utils.box import Box, get_box_covered

# Request download at:
# https://ogd.swisstopo.admin.ch/services/swiseld/services/assets/ch.swisstopo.swissalti3d/search?format=application%2Fx.ascii-xyz%2Bzip&resolution=2.0&srid=2056&state=current&csv=true
URL_LIST_PATH = 'source/swissalti3d/ch.swisstopo.swissalti3d.csv'



def fetch_and_extract(url: str)->List[Tuple[int,int,float]]:
    """
    Downloads points or reads them from cache, by reference / url.
    """
    cache.initialize_cache()

    # check if the url is already cached
    reference = utils.url_to_ref(url)
    outData = cache.get_many_from_cache(reference)
    if outData is not None:
        return outData

    print(f"downloading {reference}")
    response = requests.get(url)
    response.raise_for_status()

    outData = []

    with zipfile.ZipFile(io.BytesIO(response.content)) as z:
        with z.open(z.filelist[0]) as f:
            data = [line.split(' ') for line in f.read().decode('utf-8').splitlines()]
            for row in data[1:]:
                x = int(row[0])
                y = int(row[1])
                z = float(row[2])
                outData.append((x,y,z))

    cache.write_many_to_cache(outData, reference)

    return outData

def get_url_list(latitude_range: Tuple[int, int], longitude_range:Tuple[int, int])->List[str]:
    url_list = []
    searching_box = Box(miny=latitude_range[0],maxy=latitude_range[1], minx=longitude_range[0], maxx=longitude_range[1])
    with open(URL_LIST_PATH , 'r', newline='') as f:
        reader = csv.DictReader(f, fieldnames=['url'])
        for line in reader:
            url = line["url"]
            box_covered = get_box_covered(url)
            if searching_box.intersects(box_covered):
                url_list.append(url)
    return url_list
