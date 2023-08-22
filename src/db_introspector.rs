use sqlx::{Connection, MySqlConnection, PgConnection, Row};

/// Represents the basic structure of the INFORMATION_SCHEMA.COLUMNS table query we use
/// This table has many more columns that we do not use for the purposes of this project.
pub(crate) struct TableColumnDefinition {
    pub(crate) table_name: String,
    pub(crate) column_name: String,
    pub(crate) nullable: bool,
    pub(crate) data_type: String,
}

/// Establishes a MySQL or Postgres connection to run a single query against INFORMATION_SCHEMA.COLUMNS
/// and converts the result into a Vec<TableColumnDefinition> to later be transformed into a Vec<PythonTypedDict>
/// to later be transformed into a Python source file with the table type definitions
pub(crate) async fn get_table_definitions(
    connection_string: &str,
    schema: &str,
) -> Result<Vec<TableColumnDefinition>, anyhow::Error> {
    if connection_string.starts_with("postgres") {
        println!("Attempting to connect to provided Postgres DB.");
        let mut conn = PgConnection::connect(connection_string).await.unwrap();
        println!("Connected! Introspecting Postgres DB.");

        let query = "SELECT table_name, column_name, is_nullable, data_type FROM INFORMATION_SCHEMA.COLUMNS where table_schema = $1 order by table_name, column_name";

        let result = sqlx::query(query)
            .bind(schema)
            .fetch_all(&mut conn)
            .await?
            .iter()
            .map(|row| TableColumnDefinition {
                table_name: row.get("table_name"),
                column_name: row.get("column_name"),
                nullable: match row.get("is_nullable") {
                    "YES" => true,
                    "NO" => false,
                    _ => panic!("Unexpected value for is_nullable"),
                },
                data_type: row.get("data_type"),
            })
            .collect::<Vec<TableColumnDefinition>>();

        Ok(result)
    } else if connection_string.starts_with("mysql") {
        println!("Attempting to connect to provided MySQL DB.");
        let mut conn = MySqlConnection::connect(connection_string).await.unwrap();
        println!("Connected! Introspecting MySQL DB.");

        let query = "SELECT TABLE_NAME, COLUMN_NAME, IS_NULLABLE, DATA_TYPE FROM INFORMATION_SCHEMA.COLUMNS where TABLE_SCHEMA = ? order by TABLE_NAME, COLUMN_NAME";

        let result = sqlx::query(query)
            .bind(schema)
            .fetch_all(&mut conn)
            .await?
            .iter()
            .map(|row| TableColumnDefinition {
                table_name: row.get("TABLE_NAME"),
                column_name: row.get("COLUMN_NAME"),
                nullable: match row.get("IS_NULLABLE") {
                    "YES" => true,
                    "NO" => false,
                    _ => panic!("Unexpected value for is_nullable"),
                },
                data_type: row.get("DATA_TYPE"),
            })
            .collect::<Vec<TableColumnDefinition>>();

        Ok(result)
    } else {
        Err(anyhow::anyhow!(
            "Unsupported database type. Only MySQL and Postgres are supported."
        ))
    }
}
