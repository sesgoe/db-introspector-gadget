use sqlx::Connection;
use sqlx::MySqlConnection;
use sqlx::PgConnection;
use sqlx::Row;

#[derive(Debug)]
pub(crate) struct TableColumnDefinition {
    pub(crate) table_name: String,
    pub(crate) column_name: String,
    pub(crate) nullable: bool,
    pub(crate) data_type: String,
}

pub(crate) async fn get_table_definitions(
    connection_string: &str,
    schema: &str,
) -> Result<Vec<TableColumnDefinition>, sqlx::Error> {
    if connection_string.starts_with("postgres") {
        let mut conn = PgConnection::connect(connection_string).await.unwrap();

        let query = "SELECT table_name, column_name, is_nullable, data_type FROM information_schema.COLUMNS where table_schema = $1 order by table_name, column_name";

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
    } else {
        let mut conn = MySqlConnection::connect(connection_string).await.unwrap();

        let query = "SELECT TABLE_NAME, COLUMN_NAME, IS_NULLABLE, DATA_TYPE FROM information_schema.COLUMNS where table_schema = ? order by TABLE_NAME, COLUMN_NAME";

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
    }
}
