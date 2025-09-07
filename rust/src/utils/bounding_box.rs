use anyhow::{Error, Result};

pub struct BoundingBox {
    pub x_range: (i32, i32),
    pub y_range: (i32, i32),
}

impl BoundingBox {
    pub fn from_ranges(x_range: (i32, i32), y_range: (i32, i32)) -> Self {
        Self { x_range, y_range }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        !(
            // self is left of other
            self.x_range.1 <= other.x_range.0 ||
            // self is right of other
            self.x_range.0 >= other.x_range.1 ||
            // self is below other
            self.y_range.1 <= other.y_range.0 ||
            // self is above other
            self.y_range.0 >= other.y_range.1
        )
    }

    pub fn get_box_covered(url: &str) -> Result<Self> {
        // remove trailing '/'
        let trimmed = url.trim_end_matches('/');

        // take the last part after '/'
        let last_part = trimmed
            .rsplit('/')
            .next()
            .ok_or(Error::msg("Failed to get last path component"))?;

        // split by "_"
        let parts: Vec<&str> = last_part.split('_').collect();
        if parts.len() < 3 {
            return Err(Error::msg("Failed to extract range part from URL"));
        }

        let range_part = parts[2];
        let coords: Vec<&str> = range_part.split('-').collect();
        if coords.len() != 2 {
            return Err(Error::msg("Failed to extract easting/northing values"));
        }

        let easting: i32 = coords[0]
            .parse::<i32>()
            .map_err(|_| Error::msg("Invalid easting value"))?
            * 1000;

        let northing: i32 = coords[1]
            .parse::<i32>()
            .map_err(|_| Error::msg("Invalid northing value"))?
            * 1000;

        Ok(BoundingBox {
            x_range: (easting, easting + 1000),
            y_range: (northing, northing + 1000),
        })
    }
}
