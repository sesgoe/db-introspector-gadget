# DB Introspector Gadget

![Inspector Gadget introspecting a safe](inspector_gadget.gif)

This is a simple tool that can introspect MySQL or Postgres databases and generate a python file that contains `TypedDict` definitions for the tables and columns in the provided database schema.

> [!IMPORTANT]
> Python `TypedDict`s require a minimum Python of `3.8` for the syntax mentioned below, or Python `3.6` for backwards-compatible syntax.
> If your Python version is >= `3.6` and < `3.8` then you will need to pass the `--backwards-compat-forced` or `-b` flag

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
cargo install db-introspector-gadget
```

## Usage 🚀

#### Introspect a MySQL Database

```bash
db-introspector-gadget -c mysql://root:password@127.0.0.1:3306/testing -s testing
```

#### Introspect a Postgres Database

```bash
db-introspector-gadget -c postgres://postgres:password@localhost:5432/testing -s public
```

#### Introspect a Postgres Database and specify the output filename

```bash
db-introspector-gadget -c postgres://postgres:password@localhost:5432/testing -s public -o my_types.py
```

#### Show Help output

```bash
db-introspector-gadget --help
```

or

```bash
db-introspector-gadget -h
```

Which should output:

```bash
A MySql and Postgres database introspection tool that generates Python types

Usage: db-introspector-gadget [OPTIONS] --connection-string <CONNECTION_STRING> --schema <SCHEMA>

Options:
  -c, --connection-string <CONNECTION_STRING>
  -s, --schema <SCHEMA>
  -o, --output-filename <OUTPUT_FILENAME>      [default: table_types.py]
  -b, --backwards-compat-forced
  -h, --help                                   Print help
  -V, --version                                Print version
```
