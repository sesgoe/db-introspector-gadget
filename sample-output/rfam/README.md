# Rfam's Public MySQL Database

### What is Rfam

The Rfam database is a collection of RNA sequence families of structural RNAs including non-coding RNA genes as well as cis-regulatory elements. Each family is represented by a multiple sequence alignment and a covariance model (CM).

<https://docs.rfam.org/en/latest/about-rfam.html>

### DB Info

[DB Connection Info](https://docs.rfam.org/en/latest/database.html)

[Main Tables](https://docs.rfam.org/en/latest/database.html#main-tables)

### Example usage

```bash
db-introspector-gadget -c mysql://rfamro@mysql-rfam-public.ebi.ac.uk:4497/Rfam -s Rfam
```

### Output File

[table_types.py](./table_types.py)
