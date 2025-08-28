import sqlite3
from contextlib import closing
from typing import Optional, Tuple, List
DB_PATH = "./cache/swissalti3d.sqlite3"

initialized = False

def _get_connection():
    return sqlite3.connect(DB_PATH)


def initialize_cache():
    global initialized
    if not initialized:
        with closing(_get_connection()) as conn:
            c = conn.cursor()
            c.execute('''
                CREATE TABLE IF NOT EXISTS swissalti3d_references (
                    id TEXT NOT NULL,
                    modify_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    PRIMARY KEY (id)
                )
            ''')
            c.execute('''
                CREATE TABLE IF NOT EXISTS swissalti3d_data (
                    x INTEGER NOT NULL,
                    y INTEGER NOT NULL,
                    z FLOAT NOT NULL,
                    reference_id TEXT,
                    modify_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    PRIMARY KEY (x, y),
                    FOREIGN KEY (reference_id) REFERENCES swissalti3d_references(id)
                )
            ''')
            conn.commit()
            initialized = True

def get_from_cache(x: int,y: int)->Optional[float]:
    initialize_cache()
    with closing(_get_connection()) as conn:
        c = conn.cursor()
        c.execute('SELECT z FROM swissalti3d_data WHERE x = ? AND y = ?', (x,y))
        data = c.fetchone()
        return data[0] if data is not None else None

def check_cache(reference: str)-> Optional[Tuple[int]]:
    initialize_cache()
    with closing(_get_connection()) as conn:
        c = conn.cursor()
        c.execute('SELECT modify_at FROM swissalti3d_references WHERE id IS ?', (reference,))
        modify_at = c.fetchone()
        return modify_at

def get_many_from_cache_filtered(**kwargs)-> Optional[List[Tuple[int,int,float]]]:
    step = kwargs['step'] or 0;
    minx = kwargs['minx'];
    maxx = kwargs['maxx'];
    miny = kwargs['miny'];
    maxy = kwargs['maxy'];
    initialize_cache()
    with closing(_get_connection()) as conn:
        c = conn.cursor()
        if None not in [minx, maxx, miny, maxy]:
            c.execute('''
                SELECT x,y,z FROM swissalti3d_data
                WHERE
                    (x % ? = 1) AND (y % ? = 1)
                    AND x < ?
                    AND x > ?
                    AND y < ?
                    AND y > ?

                ORDER BY x,y ASC
                ''', (step, step, maxx, minx, maxy, miny))
        else:
            c.execute('''
                SELECT x,y,z FROM swissalti3d_data
                WHERE
                    (x % ? = 1) AND (y % ? = 1)
                    AND
                    x
                ORDER BY x,y ASC
                ''', (step, step))
        data = c.fetchall()
    return data if len(data) > 0 else None



def write_to_cache(x: int, y: int, z: float, **kwargs):
    initialize_cache()
    reference = str(kwargs.get('reference', None));
    with closing(_get_connection()) as conn:
        c = conn.cursor()
        if reference is not None:
            c.execute('INSERT OR REPLACE INTO swissalti3d_references (id) VALUES (?)', (reference,))
        c.execute('INSERT OR REPLACE INTO swissalti3d_data (x, y, z, reference_id) VALUES (?, ?, ?, ?)', (x,y,z, reference))
        conn.commit()

def write_many_to_cache(data: List[Tuple[int,int,float]], reference: str):
    initialize_cache()
    with closing(_get_connection()) as conn:
        c = conn.cursor()
        c.execute('INSERT OR REPLACE INTO swissalti3d_references (id) VALUES (?)', (reference,))
        c.executemany('INSERT OR REPLACE INTO swissalti3d_data (x, y, z, reference_id) VALUES (?, ?, ?, ?)',
            ((x, y, z, reference) for (x, y, z) in data))
        conn.commit()
