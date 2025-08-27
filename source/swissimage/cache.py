import sqlite3
from contextlib import closing
from typing import Optional, Tuple, List
DB_PATH = "./cache/swissimage.sqlite3"

initialized = False

def _get_connection():
    return sqlite3.connect(DB_PATH)


def initialize_cache():
    global initialized
    if not initialized:
        with closing(_get_connection()) as conn:
            c = conn.cursor()
            c.execute('''
                CREATE TABLE IF NOT EXISTS swissimage_references (
                    id TEXT NOT NULL,
                    modify_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    PRIMARY KEY (id)
                )
            ''')
            c.execute('''
                CREATE TABLE IF NOT EXISTS swissimage_data (
                    x INTEGER NOT NULL,
                    y INTEGER NOT NULL,
                    r INTEGER NOT NULL,
                    g INTEGER NOT NULL,
                    b INTEGER NOT NULL,
                    reference_id TEXT,
                    modify_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    PRIMARY KEY (x, y),
                    FOREIGN KEY (reference_id) REFERENCES swissimage_references(id)
                )
            ''')
            conn.commit()
            initialized = True

def get_from_cache(x: int,y: int)->Optional[Tuple[int, int, int]]:
    with closing(_get_connection()) as conn:
        c = conn.cursor()
        c.execute('SELECT r, g, b FROM swissimage_data WHERE x = ? AND y = ?', (x,y))
        data = c.fetchone()
        return (int(data[0]),int(data[1]),int(data[2])) if data is not None else None

def get_many_from_cache(reference: str)-> Optional[List[Tuple[int,int,int,int,int]]]:
    with closing(_get_connection()) as conn:
        c = conn.cursor()
        c.execute('SELECT x, y, r, g, b FROM swissimage_data WHERE reference_id IS ?', (reference,))
        data = c.fetchall()
        return data if len(data) > 0 else None

def get_many_from_cache_filtered(**kwargs)-> Optional[List[Tuple[int,int,int,int,int]]]:
    step = kwargs['step'] or 0;
    minx = kwargs['minx'];
    maxx = kwargs['maxx'];
    miny = kwargs['miny'];
    maxy = kwargs['maxy'];

    with closing(_get_connection()) as conn:
        c = conn.cursor()
        if None not in [minx, maxx, miny, maxy]:
            c.execute('''
                SELECT x,y,r,g,b FROM swissimage_data
                WHERE
                        x < ?
                    AND x > ?
                    AND y < ?
                    AND y > ?

                ORDER BY x,y ASC
                ''', (maxx, minx, maxy, miny))
        else:
            c.execute('''
                SELECT x,y,r,g,b FROM swissimage_data
                WHERE
                    (x % ? = 0) AND (y % ? = 0)
                ORDER BY x,y ASC
                ''', (step, step))
        data = c.fetchall()
    return data if len(data) > 0 else None



def write_to_cache(x: int, y: int, color: Tuple[int,int,int], **kwargs):
    reference = str(kwargs.get('reference', None));
    with closing(_get_connection()) as conn:
        c = conn.cursor()
        if reference is not None:
            c.execute('INSERT OR REPLACE INTO swissimage_references (id) VALUES (?)', (reference,))
        c.execute('INSERT OR REPLACE INTO swissimage_data (x, y, r, g, b, reference_id) VALUES (?, ?, ?, ?, ?, ?)', (x,y,*color, reference))
        conn.commit()

def write_many_to_cache(data: List[Tuple[Tuple[int,int],Tuple[int,int,int]]], reference: str):
    with closing(_get_connection()) as conn:
        c = conn.cursor()
        c.execute('INSERT OR REPLACE INTO swissimage_references (id) VALUES (?)', (reference,))
        c.executemany('INSERT OR REPLACE INTO swissimage_data (x, y, r, g, b, reference_id) VALUES (?, ?, ?, ?, ?, ?)',
            ((x, y, int(r), int(g), int(b), reference) for ((x, y), (r,g,b)) in data))
        conn.commit()
