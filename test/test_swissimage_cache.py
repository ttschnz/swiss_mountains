import unittest
import os
from random import randint, choice
from source.swissimage import cache

TEST_DB = "./cache/test.sqlite3"

class TestSwissImageCache(unittest.TestCase):
    def setUp(self):
        # Use a separate test database
        if os.path.exists(TEST_DB):
            os.remove(TEST_DB)
        # Patch your cache module to use TEST_DB if needed
        # For example, set cache.DB_PATH = TEST_DB
        cache.DB_PATH = TEST_DB
        cache.initialize_cache()

    def tearDown(self):
        if os.path.exists(TEST_DB):
            os.remove(TEST_DB)

    def test_write_and_read(self):

        # Generate random data
        data = [
                (
                    randint(int(1e6),int(1e7)),
                    randint(int(1e6),int(1e7)),
                    (
                        randint(1,255),
                        randint(1,255),
                        randint(1,255)
                    ),
                ) for _ in range(100)]

        for row in data:
            cache.write_to_cache(*row)

        for _ in range(50):
            row = choice(data)
            color = cache.get_from_cache(row[0],row[1]);
            self.assertEqual(color, row[2])

    def test_cache_miss(self):
        result = cache.get_from_cache(0,0)
        self.assertIsNone(result)

if __name__ == '__main__':
    unittest.main()
