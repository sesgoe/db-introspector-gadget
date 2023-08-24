# DB Introspector Gadget

![Inspector Gadget introspecting a safe](inspector_gadget.gif)

This is a CLI tool that can introspect MySQL or Postgres databases and generate a Python source file that contains `TypedDict` definitions for the tables and columns in the provided database schema.

> [!IMPORTANT]
> This tool generates Python source code that requires Python >= `3.10` by default.

> [!NOTE]
> You can use the `--minimum-python-version` (`-p`) flag to change this. See the help documentation below for further clarification.

The intention of this tool is to help make it easier to write type-safe python code that interacts with databases.

If you have some example Python code that looks like this:

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

This tool can generate a Python file that looks like this:

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

<https://crates.io/crates/db-introspector-gadget>

## Sample Outputs

You can check inside the [sample-output](./sample-output/) folder to find a [MySQL](./sample-output/rfam/README.md) and [Postgres](./sample-output/rna-central/README.md) example.

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
          The MySQL or Postgres connection string in the format `mysql://___` or `postgres://___` of the database that you would like to introspect
  -s, --schema <SCHEMA>
          The database schema that you would like to introspect and create table types for
  -o, --output-filename <OUTPUT_FILENAME>
          Optional output file path for the final source file output [default: table_types.py]
  -p, --minimum-python-version <MINIMUM_PYTHON_VERSION>
          Establishes the minimum supported Python Version [default: python3-10] [possible values: python3-6, python3-8, python3-10]
  -h, --help
          Print help (see more with '--help')
  -V, --version
          Print version
```
