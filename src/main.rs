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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    connection_string: String,

    #[arg(short, long)]
    schema: String,

    #[arg(short, long, default_value = "table_types.py")]
    output_filename: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let table_definitions: Vec<TableColumnDefinition> =
        get_table_definitions(&args.connection_string, &args.schema)
            .await
            .context("Unable to connect to database")?;

    let python_dicts = convert_table_column_definitions_to_python_dicts(table_definitions);
    let file_contents = write_python_dicts_to_str(python_dicts);

    let filename = args
        .output_filename
        .unwrap_or(String::from("table_types.py"));

    let mut file =
        fs::File::create(&filename).context(format!("Unable to create {} file.", &filename))?;
    file.write_all(file_contents.as_bytes())?;

    println!("Successfully created {}", &filename);

    Ok(())
}
