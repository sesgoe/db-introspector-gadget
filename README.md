# Rust DB Introspector

This is a simple tool that can introspect MySQL or Postgres databases and generate a python file that contains `TypedDict` definitions for the tables and columns in the provided database schema.

The intention is to help make it easier to write type-safe python code that interacts with databases.

If you have some example python code that looks like this:

```python
import psycopg2

with psycopg2.connect("dbname=testing user=postgres password=password") as conn:
    with conn.cursor() as cur:
        cur.execute("SELECT * FROM users")
        for row in cur.fetchall():
            print(row)
```

If you assume for a moment that the `users` table has the following schema:

```sql
CREATE TABLE users(
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

This tool can generate a python file that looks like this:

```python
import datetime
from typing import TypedDict

class Users(TypedDict):
    id: int
    name: str
    email: str
    created_at: datetime.datetime
```

So that you can improve your code to look like this:

```python
import psycopg2
from my_types import Users

with psycopg2.connect("dbname=testing user=postgres password=password") as conn:
    with conn.cursor() as cur:
        cur.execute("SELECT * FROM users")
        users: List[Users] = cur.fetchall()
        for row in users:
            print(user) # type-hinting is now available on this dict!
```

## Installation 🛠️

Note: It's a prerequisite that you have `cargo` installed. If you don't, you can install it [here](https://www.rust-lang.org/tools/install).

```bash
cargo install db-introspector
```

## Usage 🚀

#### Introspect a MySQL Database

```bash
db-introspector -c mysql://root:password@127.0.0.1:3306/testing -s testing
```

#### Introspect a Postgres Database

```bash
db-introspector -c postgres://postgres:password@localhost:5432/testing -s public
```

#### Introspect a Postgres Database and specify the output filename

```bash
db-introspector -c postgres://postgres:password@localhost:5432/testing -s public -o my_types.py
```

#### Show Help output

```bash
db-introspector --help
```

or

```bash
db-introspector -h
```
