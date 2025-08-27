
from dataclasses import dataclass
from typing import Self

@dataclass
class Box:
    minx: float
    miny: float
    maxx: float
    maxy: float

    def intersects(self, other: Self)->bool:
        return not (
                    self.maxx <= other.minx or  # self is left of other
                    self.minx >= other.maxx or  # self is right of other
                    self.maxy <= other.miny or  # self is below other
                    self.miny >= other.maxy     # self is above other
                )

def get_box_covered(url: str) -> Box:
    """
    Extracts easting and northing values from a URL and returns a Box describing the area covered by the zip.
    """
    try:
        last_part = url.rstrip('/').split('/')[-1]
        parts = last_part.split('_')
        if len(parts) < 3:
            raise ValueError("Failed to extract range part from URL")
        range_part = parts[2]
        coords = range_part.split('-')
        if len(coords) != 2:
            raise ValueError("Failed to extract easting/northing values")
        easting = float(coords[0]) * 1000.0
        northing = float(coords[1]) * 1000.0

        return Box(easting, northing, easting + 1000, northing + 1000)
    except Exception as e:
        raise ValueError(f"Error parsing URL: {e}")
