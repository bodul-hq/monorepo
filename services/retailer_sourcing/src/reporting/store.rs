//! Statistics for ordinary tables in this connection's active schema.
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Text};
use serde::Serialize;

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub(super) enum RowCountKind {
    Estimated,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub(super) struct DatabaseTables {
    pub schema_name: String,
    pub generated_at: String,
    pub row_count_kind: RowCountKind,
    pub tables: Vec<TableStats>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub(super) struct TableStats {
    pub table_name: String,
    /// Estimated live rows from PostgreSQL statistics; may lag behind writes.
    pub row_count: u64,
    /// Table storage including TOAST, free space and visibility maps.
    pub table_bytes: u64,
    /// Storage of indexes attached to this table.
    pub index_bytes: u64,
    /// Sum of table_bytes and index_bytes.
    pub total_bytes: u64,
}

#[derive(QueryableByName)]
struct TableRecord {
    #[diesel(sql_type = Text)]
    table_name: String,
    #[diesel(sql_type = BigInt)]
    row_count: i64,
    #[diesel(sql_type = BigInt)]
    table_bytes: i64,
    #[diesel(sql_type = BigInt)]
    index_bytes: i64,
}

pub(super) fn load(connection: &mut PgConnection) -> QueryResult<DatabaseTables> {
    #[derive(QueryableByName)]
    struct Schema {
        #[diesel(sql_type = Text)]
        schema_name: String,
    }
    let schema: Schema =
        diesel::sql_query("SELECT current_schema()::text AS schema_name").get_result(connection)?;
    let records: Vec<TableRecord> =
        diesel::sql_query(include_str!("database_tables.sql")).load(connection)?;
    let tables = records
        .into_iter()
        .map(|record| {
            let unsigned = |value: i64| {
                u64::try_from(value)
                    .map_err(|e| diesel::result::Error::DeserializationError(Box::new(e)))
            };
            let table_bytes = unsigned(record.table_bytes)?;
            let index_bytes = unsigned(record.index_bytes)?;
            Ok(TableStats {
                table_name: record.table_name,
                row_count: unsigned(record.row_count)?,
                table_bytes,
                index_bytes,
                total_bytes: table_bytes + index_bytes,
            })
        })
        .collect::<QueryResult<Vec<_>>>()?;
    Ok(DatabaseTables {
        schema_name: schema.schema_name,
        generated_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        row_count_kind: RowCountKind::Estimated,
        tables,
    })
}
