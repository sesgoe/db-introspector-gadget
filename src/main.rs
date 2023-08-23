//! `db-introspector-gadget` is a cli rust tool to introspect a MySQL or Postgres Database
//! and create a Python source code output file that contains `TypedDict` definitions for all tables
//! introspected in the supplied schema.
//!
//! By default this tool outputs Python source files that require Python >= 3.10, but the tool supports
//! a minimum Python version of 3.6, 3.8, and 3.10

#![deny(unsafe_code)]

use std::{fs, io::Write, path::PathBuf};

use anyhow::Context;
use clap::Parser;

mod db_introspector;
use db_introspector::{get_table_definitions, TableColumnDefinition};
use python_type_file_writer::{
    convert_table_column_definitions_to_python_dicts, write_python_dicts_to_str,
};

mod python_type_file_writer;
mod python_types;

/// Defines the minimum supported Python version for the source file output.
/// `TypedDict` definitions look different in each of these versions of python.
///
/// In Python 3.6:
/// ```python
/// SomeDictionary = TypedDict('SomeDictionary', {
///     'some_property': Optional[str]
/// })
/// ```
///
/// In Python 3.8
/// ```python
/// class SomeDictionary(TypedDict):
///     some_property: Optional[str]
/// ```
///
/// In Python 3.10
/// ```python
/// class SomeDictionary(TypedDict):
///     some_property: str | None
/// ```
#[derive(Debug, Copy, clap::ValueEnum, PartialEq, Eq, Clone)]
enum MinimumPythonVersion {
    Python3_6,
    Python3_8,
    Python3_10,
}

/// This is a `clap` struct to define the arguments this tool takes in as input.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The MySQL or Postgres connection string in the format `mysql://___` or `postgres://___`
    /// of the database that you would like to introspect
    #[arg(short, long)]
    connection_string: String,

    /// The database schema that you would like to introspect and create table types for
    #[arg(short, long)]
    schema: String,

    /// Optional output file path for the final source file output
    #[arg(short, long, default_value = "table_types.py")]
    output_filename: Option<PathBuf>,

    /// Establishes the minimum supported Python Version
    ///
    /// Python 3.6 requires the backward-compat syntax and `Optional[T]`
    ///
    /// Python 3.8 allows for modern syntax and `Optional[T]`
    ///
    /// Python 3.10 allows for modern syntax and `T | None`
    #[arg(short='p', long, value_enum, default_value_t = MinimumPythonVersion::Python3_10)]
    minimum_python_version: MinimumPythonVersion,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let table_definitions: Vec<TableColumnDefinition> =
        get_table_definitions(&args.connection_string, &args.schema)
            .await
            .context("Unable to connect to database")?;

    let python_typed_dicts = convert_table_column_definitions_to_python_dicts(table_definitions);
    let file_contents = write_python_dicts_to_str(python_typed_dicts, args.minimum_python_version);

    let file_path = args
        .output_filename
        .unwrap_or(String::from("table_types.py").into());

    let mut file = fs::File::create(&file_path).context(format!(
        "Unable to create {} file.",
        &file_path.to_string_lossy()
    ))?;
    file.write_all(file_contents.as_bytes())?;

    println!("Successfully created {}", &file_path.to_string_lossy());

    Ok(())
}
