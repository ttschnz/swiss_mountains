import requests
import logging
import io
import csv
from PIL import Image
import numpy

from typing import List, Tuple, Self

from . import cache
import utils
from utils.box import Box, get_box_covered

# Request download at
# https://ogd.swisstopo.admin.ch/services/swiseld/services/assets/ch.swisstopo.swissimage-dop10/search?format=image%2Ftiff%3B%20application%3Dgeotiff%3B%20profile%3Dcloud-optimized&resolution=2.0&srid=2056&state=current&csv=true
URL_LIST_PATH = 'source/swissimage/ch.swisstopo.swissimage.csv'


def fetch_and_extract(url: str)->List[Tuple[Tuple[int,int],Tuple[int, int, int]]]:
    """
    Downloads points or reads them from cache, by reference / url.
    """
    cache.initialize_cache()

    # check if the url is already cached
    reference = utils.url_to_ref(url)
    cached_data = cache.get_many_from_cache(reference)
    if cached_data is not None: # outData: Option[List[Tuple[int, int, int, int, int]]]
        return [((x,y),(r,g,b)) for (x,y,r,g,b) in cached_data]

    print(f"downloading {reference}")
    response = requests.get(url)
    response.raise_for_status()

    out_data:List[Tuple[Tuple[int, int], Tuple[int, int, int]]]  = []

    box = get_box_covered(url)
    check_map = set();
    with Image.open(io.BytesIO(response.content)) as im:
        imarray = numpy.array(im)
        for yx in numpy.ndindex(imarray.shape[:2]):
            x = int(box.minx + yx[1]*2)
            y = int(box.maxy - yx[0]*2)
            color = imarray[yx]
            if (x,y) not in check_map:
                check_map.add((x,y))
                out_data.append(((x,y), tuple(color)))

    cache.write_many_to_cache(out_data, reference)

    return out_data

def get_url_list(latitude_range: Tuple[int, int], longitude_range:Tuple[int, int])->List[str]:
    url_list = []
    searching_box = Box(minx=longitude_range[0], maxx=longitude_range[1], miny=latitude_range[0],maxy=latitude_range[1])
    with open(URL_LIST_PATH , 'r', newline='') as f:
        reader = csv.DictReader(f, fieldnames=['url'])
        for line in reader:
            url = line["url"]
            box_covered = get_box_covered(url)
            if searching_box.intersects(box_covered):
                url_list.append(url)
    return url_list
