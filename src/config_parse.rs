use serde::Deserialize;
use std::collections::HashMap;

use std::fs;

#[derive(Debug, Default, Deserialize)]
pub struct Conf {
    #[serde(default = "default_version")]
    version: String,
    tables: HashMap<String, Table>,
}

// exmaple: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=4627bb01fdc7b7811e8478d0bf070782
fn default_version() -> String {
    "v1".to_string()
}

#[derive(Debug, Deserialize, Clone)]
struct Table {
    columns: HashMap<String, Column>,
}

#[derive(Debug, Deserialize, Clone)]
struct Column {
    datatype: ColumnDataType,
}

#[derive(Debug, Deserialize, Clone)]
enum ColumnDataType {
    Integer,
    String,
    Float,
    DateTime
}

pub struct ConfigParser {
    config: Conf
}

impl ConfigParser {

    pub fn new(conf: Conf) -> ConfigParser {
        ConfigParser {
            config: conf
        }
    }

    // HANDLE BASE OBJECT CREATION METHODS
    fn match_to_sql_type(&self, d: &ColumnDataType) -> &str {
        match d {
            ColumnDataType::String => ": Union[str, None] = Field(default=None, nullable=True)",
            ColumnDataType::Integer => ": Union[int, None] = Field(default=None, nullable=True)",
            ColumnDataType::Float => ": Union[float, None] = Field(default=None, nullable=True)",
            ColumnDataType::DateTime => ": Union[datetime.datetime, None] = Field(default_factory=datetime.datetime.now, nullable=True)"
        }
    }

    fn generate_column(&self, name: &str, col: &Column) -> String {
        let d: &str = self.match_to_sql_type(&col.datatype);
        return format!("\n\t{name}{d}");
    }

    fn generate_base_class_name(&self, name: &str) -> String {
        let formatted_name: String = format!("{name}").to_uppercase();
        return format!("{formatted_name}_Base");
    }

    fn generate_base_class_object(&self, table_name: &str, table: Table) -> String {
        let base_name: String = self.generate_base_class_name(table_name);
        let columns: String = table
        .columns
        .into_iter()
        .map(|(name, column)| self.generate_column(&name, &column))
        .collect();
        return format!(
            "class {base_name}(SQLModel):{columns}"
        );
    }

    fn create_base_model(&self) -> String {
        let base_tables = self
            .config
            .tables
            .clone()
            .into_iter()
            .map(
                |(table_name, table)|
                self.generate_base_class_object(&table_name, table)
            );
        let base_tables_2 = base_tables.collect();
        return base_tables_2;
    }

    // HANDLE REGULAR CLASS METHODS
    fn generate_regular_class_name(&self, name: &str) -> String {
        let formatted_name: String = format!("{name}").to_uppercase();
        return format!("{formatted_name}");
    }

    fn generate_regular_class_object(&self, table_name: &str) -> String {
        let base_regular_name: String = self.generate_regular_class_name(table_name);
        let base_object_name: String = self.generate_base_class_name(table_name);
        return format!(
            "class {base_regular_name}({base_object_name}, table=True):\n\tid: int | None = Field(default=None, primary_key=True)"
        );
    }

    fn create_regular_model(&self) -> String {
        let base_tables = self
            .config
            .tables
            .clone()
            .into_iter()
            .map(
                |(table_name, _)|
                self.generate_regular_class_object(&table_name)
            );
        let base_tables_2 = base_tables.collect();
        return base_tables_2
    }

    // HANDLE CLASS UPDATE METHODS
    fn match_to_sql_type_update(&self, d: &ColumnDataType) -> &str {
        match d {
            ColumnDataType::String => ": Union[str, None] = None",
            ColumnDataType::Integer => ": Union[int, None] = None",
            ColumnDataType::Float => ": Union[float, None] = None",
            ColumnDataType::DateTime => ": Union[datetime.datetime, None] = None"
        }
    }

    fn generate_column_update(&self, name: &str, col: &Column) -> String {
        let d: &str = self.match_to_sql_type_update(&col.datatype);
        return format!("\n\t{name}{d}");
    }

    fn generate_update_class_name(&self, name: &str) -> String {
        let formatted_name: String = format!("{name}").to_uppercase();
        return format!("{formatted_name}_Update");
    }

    fn generate_update_class_object(&self, table_name: &str, table: Table) -> String {
        let base_update_name: String = self.generate_update_class_name(table_name);
        let base_object_name: String = self.generate_base_class_name(table_name);
        let columns: String = table
        .columns
        .into_iter()
        .map(|(name, column)| self.generate_column_update(&name, &column))
        .collect();
        return format!(
            "class {base_update_name}({base_object_name}):{columns}"
        );
    }

     fn create_update_model(&self) -> String {
        let base_tables = self
            .config
            .tables
            .clone()
            .into_iter()
            .map(
                |(table_name, table)|
                self.generate_update_class_object(&table_name, table)
            );
        let base_tables_2 = base_tables.collect();
        return base_tables_2;
    }

    // MODEL IMPORTS
    fn model_imports(&self) -> String {
        return "import datetime\nfrom typing import Union\nfrom sqlmodel import Field, SQLModel".to_owned()
    }

    // CREATE ALL MODELS
    fn create_models(&self) -> String {
        let imports: String = self.model_imports();
        let base_model: String = self.create_base_model();
        let regular_model: String = self.create_regular_model();
        let update_model: String = self.create_update_model();
        return format!("{imports}\n{base_model}\n{regular_model}\n{update_model}")
    }
    
    // --------------------------------------

    // CREATE API
    fn generate_api(&self, table_name: &str) -> String {
        let update_object_name: String = self.generate_update_class_name(table_name);
        let base_object_name: String = self.generate_base_class_name(table_name);
        let regular_object_name: String = self.generate_regular_class_name(table_name);
        format!("

{table_name} = APIRouter(
    prefix='/{table_name}',
    tags=['{table_name}'],
    responses={{404: {{'description': 'Not found'}} }},
)


@{table_name}.get('/')
async def view_all(
    session: SessionDep,
    offset: int = 0,
    limit: Annotated[int, Query(le=100)] = 100
) -> List[{regular_object_name}]:
    models = session.exec(select({regular_object_name}).offset(offset).limit(limit)).all()
    return models


@{table_name}.post('/create', response_model={regular_object_name})
async def create_one(model: {base_object_name}, session: SessionDep) -> {regular_object_name}:
    db_model = {regular_object_name}.model_validate(model)
    session.add(db_model)
    session.commit()
    session.refresh(db_model)
    return db_model


@{table_name}.get('/{{pk}}')
async def view_one(pk: int, session: SessionDep) -> {regular_object_name}:
    model = session.get({regular_object_name}, pk)
    if not model:
        raise HTTPException(status_code=404, detail='Not found')
    return model


@{table_name}.delete('/{{pk}}')
async def delete_one(pk: int, session: SessionDep) -> JSONResponse:
    model = session.get({regular_object_name}, pk)
    if not model:
        raise HTTPException(status_code=404, detail='Not found')
    session.delete(model)
    session.commit()
    return JSONResponse(content={{'ok': True}})


@{table_name}.patch('/{{pk}}')
async def update_one(pk: int, model: {update_object_name}, session: SessionDep) -> {regular_object_name}:
    model_db = session.get({regular_object_name}, pk)
    if not model:
        raise HTTPException(status_code=404, detail='Not found')
    model_data = model.model_dump(exclude_unset=True)
    model_db.sqlmodel_update(model_data)
    session.add(model_db)
    session.commit()
    session.refresh(model_db)
    return model

app.include_router({table_name})
")
    }

    fn create_api(&self) -> String {
        let base_tables = self
            .config
            .tables
            .clone()
            .into_iter()
            .map(
                |(table_name, _)|
                self.generate_api(&table_name)
            );
        let base_tables_2: String = base_tables.collect();
        return format!("
from fastapi import FastAPI
from db import create_db_and_tables
from typing import Annotated, List
from fastapi import APIRouter, HTTPException, Query
from fastapi.responses import JSONResponse
from sqlmodel import select

from db import SessionDep
from models import *

app = FastAPI()

@app.on_event('startup')
def on_startup():
    create_db_and_tables()

@app.get('/')
async def root():
    return {{'message': 'Hello World'}}
{base_tables_2}
        "
        );
    }

    // CREATE DATABASE SETUP
    fn create_database(&self) -> String {
        return format!("
from typing import Annotated

from fastapi import Depends
from sqlmodel import Session, SQLModel, create_engine

#### Tables
from models import *

#### Configuration
sqlite_file_name = 'database.db'
sqlite_url = f'sqlite:///{{sqlite_file_name}}'

connect_args = {{'check_same_thread': False}}
engine = create_engine(sqlite_url, connect_args=connect_args)


def create_db_and_tables():
    SQLModel.metadata.create_all(engine)


def get_session():
    with Session(engine) as session:
        yield session


SessionDep = Annotated[Session, Depends(get_session)]
        ")

    }

    // entrypoint
    pub fn parse(&self) {
        println!("Using Symbiotic Version: {}", self.config.version);
        let models: String = self.create_models();
        let api: String = self.create_api();
        let db: String = self.create_database();
        fs::write("./app/models.py", models).expect("Unable to write models file");
        fs::write("./app/api.py", api).expect("Unable to write API file");
        fs::write("./app/db.py", db).expect("Unable to write database file");
    }
}
