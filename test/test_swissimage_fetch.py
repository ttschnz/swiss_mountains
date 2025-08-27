import unittest
import os
import csv
from random import randint, choice
from source import swissimage
from source.swissimage import fetch

TEST_DB = "./cache/test.sqlite3"

class TestSwissImageFetch(unittest.TestCase):
    def setUp(self):
        # Use a separate test database
        if os.path.exists(TEST_DB):
            os.remove(TEST_DB)
        # Patch your cache module to use TEST_DB if needed
        # For example, set cache.DB_PATH = TEST_DB
        swissimage.cache.DB_PATH = TEST_DB
        swissimage.cache.initialize_cache()

    def tearDown(self):
        if os.path.exists(TEST_DB):
            os.remove(TEST_DB)

    def test_fetch(self):
        with open(swissimage.fetch.URL_LIST_PATH , 'r', newline='') as f:
            reader = csv.DictReader(f, fieldnames=['url'])
            lines = list(reader)
            for _ in range(5):
                line = choice(lines)
                data = swissimage.fetch.fetch_and_extract(line['url'])
                for _ in range(5):
                    ((x,y),color_direct) = choice(data)
                    color_cached = swissimage.cache.get_from_cache(x,y)
                    self.assertEqual(color_direct, color_cached)
                    self.assertEqual(swissimage.fetch.get_url_list((y,y),(x,x)),[line['url']])

if __name__ == '__main__':
    unittest.main()
