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
                modify_at = swissimage.cache.check_cache(line['url'])
                self.assertIsNotNone(modify_at)

if __name__ == '__main__':
    unittest.main()
