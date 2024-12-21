# Symbiotic
simple yaml micro backend

Just for quickly setting up a CRUD API Server for development purposes.

Essentially setups up each table in SQLite and creates all the CRUD routes for each table.


Setup:

```
version: v1
tables:
  table_one: # change to any name underscored
    columns:
        column_one: # change to any name underscored
            datatype: String
        column_two:
            datatype: String
        column_three:
            datatype: Integer
  users:
    columns:
        name: # change to any name underscored
            datatype: String
        date:
            datatype: Datetime
        count:
            datatype: Integer
```

Create Server:

- Fill out YAML file
- Then in your terminal:

```
cargo run
```

Setup venv for API Server

```
python -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

Start Server:

```
fastapi dev ./app/api.py
```

Go to localhost to see all routes:
```
http://0.0.0.0:8000/docs 
```
