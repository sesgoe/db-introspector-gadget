//! `db-introspector-gadget is a cli rust tool to introspect a MySQL or Postgres Database
//! and create a Python source code output file that contains `TypedDict` definitions for all tables
//! introspected in the supplied schema.
//!
//! By default this tool outputs Python source files that require Python >= 3.8.
//!
//! You can use the `--backwards-compat-forced` (or `-b`) flag to use an alternative syntax that
//! supports Python >= 3.6.

#![deny(unsafe_code)]

use std::{fs, io::Write};

use anyhow::Context;
use clap::Parser;

mod db_introspector;
use db_introspector::{get_table_definitions, TableColumnDefinition};
use python_type_file_writer::{
    convert_table_column_definitions_to_python_dicts, write_python_dicts_to_str,
};

mod python_type_file_writer;
mod python_types;

/// This is a `clap` struct to define the arguments this tool takes in as input.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The MySQL or Postgres connection string in the format mysql://___ or postgres://___
    /// of the database that you would like to introspect
    #[arg(short, long)]
    connection_string: String,

    /// The database schema that you would like to introspect and create table types for
    #[arg(short, long)]
    schema: String,

    /// The Python source filename for output. Defaults to `table_types.py`
    #[arg(short, long, default_value = "table_types.py")]
    output_filename: Option<String>,

    /// If you need to support Python >= 3.6 and < 3.8, you will need to use this
    /// flag to force-enable the alternative, backward-compatible syntax
    #[arg(short, long, default_value = "false")]
    backwards_compat_forced: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let table_definitions: Vec<TableColumnDefinition> =
        get_table_definitions(&args.connection_string, &args.schema)
            .await
            .context("Unable to connect to database")?;

    let python_typed_dicts = convert_table_column_definitions_to_python_dicts(table_definitions);
    let file_contents = write_python_dicts_to_str(python_typed_dicts, args.backwards_compat_forced);

    let filename = args
        .output_filename
        .unwrap_or(String::from("table_types.py"));

    let mut file =
        fs::File::create(&filename).context(format!("Unable to create {} file.", &filename))?;
    file.write_all(file_contents.as_bytes())?;

    println!("Successfully created {}", &filename);

    Ok(())
}
