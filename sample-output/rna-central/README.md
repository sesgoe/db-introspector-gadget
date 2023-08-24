# RNA Central's Public Postgres Database

### What is RNACentral?

RNAcentral is an open public resource that offers integrated access to a comprehensive and up-to-date set of ncRNA sequences.

RNAcentral assigns identifiers to distinct ncRNA sequences and automatically updates links between sequences and identifiers maintained by expert databases.

<https://rnacentral.org/help#what-is-rnacentral>

### DB Info

[DB Connection Info](https://rnacentral.org/help/public-database)

[DB Schema](https://rnacentral.org/static/img/rnacentral_latest_schema.png)

### Example usage

```bash
db-introspector-gadget -c postgresql://reader:NWDMCE5xdipIjRrp@hh-pgsql-public.ebi.ac.uk:5432/pfmegrnargs -s rnacen
```

### Output File

[table_types.py](./table_types.py)
