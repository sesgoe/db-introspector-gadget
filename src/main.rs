use clap::Parser;

mod db_introspector;
use db_introspector::{get_table_definitions, TableColumnDefinition};

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
async fn main() {
    let args = Args::parse();

    let table_definitions: Vec<TableColumnDefinition> =
        get_table_definitions(&args.connection_string, &args.schema)
            .await
            .unwrap();

    println!("{table_definitions:?}");
}
