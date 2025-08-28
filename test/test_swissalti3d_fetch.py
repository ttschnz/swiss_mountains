import unittest
import os
import csv
from random import randint, choice
from source import swissalti3d
from source.swissalti3d import fetch

TEST_DB = "./cache/test.sqlite3"

class TestSwissAlti3DFetch(unittest.TestCase):
    def setUp(self):
        # Use a separate test database
        if os.path.exists(TEST_DB):
            os.remove(TEST_DB)
        # Patch your cache module to use TEST_DB if needed
        # For example, set cache.DB_PATH = TEST_DB
        swissalti3d.cache.DB_PATH = TEST_DB
        swissalti3d.cache.initialize_cache()

    def tearDown(self):
        if os.path.exists(TEST_DB):
            os.remove(TEST_DB)

    def test_fetch(self):
        with open(swissalti3d.fetch.URL_LIST_PATH , 'r', newline='') as f:
            reader = csv.DictReader(f, fieldnames=['url'])
            lines = list(reader)
            for _ in range(5):
                line = choice(lines)
                modify_at = swissalti3d.cache.check_cache(line['url'])
                self.assertIsNotNone(modify_at)

if __name__ == '__main__':
    unittest.main()
