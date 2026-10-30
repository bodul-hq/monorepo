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

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub(super) struct EntryStatuses {
    pub generated_at: String,
    /// Inclusive scheduled_at bound, at UTC midnight.
    pub window_start: String,
    /// Exclusive scheduled_at bound, at the next UTC midnight.
    pub window_end: String,
    pub inbox_entries: Vec<StatusCount>,
    pub outbox_entries: Vec<StatusCount>,
    pub command_entries: Vec<StatusCount>,
    pub event_entries: Vec<StatusCount>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub(super) struct StatusCount {
    pub status: i32,
    pub status_text: &'static str,
    /// Exact number of entries at the query snapshot.
    pub count: u64,
}

pub(super) fn load_entry_statuses(connection: &mut PgConnection) -> QueryResult<EntryStatuses> {
    load_entry_statuses_at(connection, chrono::Utc::now())
}

pub(super) fn load_entry_statuses_at(
    connection: &mut PgConnection,
    now: chrono::DateTime<chrono::Utc>,
) -> QueryResult<EntryStatuses> {
    let start = now.date_naive().and_time(chrono::NaiveTime::MIN).and_utc();
    let end = start + chrono::Duration::days(1);
    #[derive(QueryableByName)]
    struct Record {
        #[diesel(sql_type = Text)]
        kind: String,
        #[diesel(sql_type = diesel::sql_types::Integer)]
        status: i32,
        #[diesel(sql_type = BigInt)]
        count: i64,
    }
    // One statement gives all four queues the same PostgreSQL snapshot.
    let records: Vec<Record> = diesel::sql_query(
        "SELECT 'command' AS kind, status, COUNT(*) AS count FROM command_entries WHERE scheduled_at >= $1 AND scheduled_at < $2 GROUP BY status
         UNION ALL
         SELECT 'event' AS kind, status, COUNT(*) AS count FROM event_entries WHERE scheduled_at >= $1 AND scheduled_at < $2 GROUP BY status
         UNION ALL
         SELECT 'inbox' AS kind, status, COUNT(*) AS count FROM inbox_entries WHERE scheduled_at >= $1 AND scheduled_at < $2 GROUP BY status
         UNION ALL
         SELECT 'outbox' AS kind, status, COUNT(*) AS count FROM outbox_entries WHERE scheduled_at >= $1 AND scheduled_at < $2 GROUP BY status
         ORDER BY kind, status",
    )
    .bind::<diesel::sql_types::Timestamptz, _>(start)
    .bind::<diesel::sql_types::Timestamptz, _>(end)
    .load(connection)?;
    let mut report = EntryStatuses {
        generated_at: now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        window_start: start.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        window_end: end.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        inbox_entries: Vec::new(),
        outbox_entries: Vec::new(),
        command_entries: Vec::new(),
        event_entries: Vec::new(),
    };
    for record in records {
        // Mulac 0.3.0 CommandStatus and EventStatus share these persisted codes.
        let status_text = match record.status {
            0 => "received",
            2 => "reserved",
            4 => "failed",
            5 => "completed",
            7 => "dead",
            8 => "archive",
            _ => "unknown",
        };
        let entry = StatusCount {
            status: record.status,
            status_text,
            count: u64::try_from(record.count)
                .map_err(|e| diesel::result::Error::DeserializationError(Box::new(e)))?,
        };
        match record.kind.as_str() {
            "inbox" => report.inbox_entries.push(entry),
            "command" => report.command_entries.push(entry),
            "event" => report.event_entries.push(entry),
            "outbox" => report.outbox_entries.push(entry),
            _ => unreachable!("queue kinds are literals in the reporting query"),
        }
    }
    Ok(report)
}

#[cfg(test)]
mod postgres_tests {
    use super::*;
    use diesel::connection::SimpleConnection;

    #[test]
    #[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
    fn postgres_daily_window_and_rescheduling() {
        let url = std::env::var("DATABASE_URL_TEST").expect("DATABASE_URL_TEST");
        let mut conn = PgConnection::establish(&url).unwrap();
        conn.batch_execute("CREATE TEMP TABLE command_entries(status integer, scheduled_at timestamptz, received_at timestamptz DEFAULT '2000-01-01Z'); CREATE TEMP TABLE event_entries(LIKE command_entries INCLUDING DEFAULTS); CREATE TEMP TABLE inbox_entries(LIKE command_entries INCLUDING DEFAULTS); CREATE TEMP TABLE outbox_entries(LIKE command_entries INCLUDING DEFAULTS); SET TIME ZONE 'America/New_York';").unwrap();
        for date in ["2026-03-08T12:00:00Z", "2026-12-31T23:59:59Z"] {
            let now = chrono::DateTime::parse_from_rfc3339(date)
                .unwrap()
                .with_timezone(&chrono::Utc);
            let start = now.date_naive().and_time(chrono::NaiveTime::MIN).and_utc();
            let end = start + chrono::Duration::days(1);
            conn.batch_execute(
                "TRUNCATE command_entries, event_entries, inbox_entries, outbox_entries",
            )
            .unwrap();
            assert!(
                load_entry_statuses_at(&mut conn, now)
                    .unwrap()
                    .command_entries
                    .is_empty()
            );
            for table in [
                "command_entries",
                "event_entries",
                "inbox_entries",
                "outbox_entries",
            ] {
                diesel::sql_query(format!("INSERT INTO {table}(status,scheduled_at) VALUES (0,$1 - interval '1 microsecond'),(0,$1),(5,$1 + interval '23 hours'),(99,$2 - interval '1 microsecond'),(0,$2)"))
                    .bind::<diesel::sql_types::Timestamptz,_>(start)
                    .bind::<diesel::sql_types::Timestamptz,_>(end)
                    .execute(&mut conn).unwrap();
            }
            let report = load_entry_statuses_at(&mut conn, now).unwrap();
            assert_eq!(
                report.window_start,
                start.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
            );
            assert_eq!(
                report.window_end,
                end.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
            );
            assert_eq!(
                report.generated_at,
                now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
            );
            for entries in [
                &report.command_entries,
                &report.event_entries,
                &report.inbox_entries,
                &report.outbox_entries,
            ] {
                assert_eq!(
                    entries
                        .iter()
                        .map(|e| (e.status, e.count))
                        .collect::<Vec<_>>(),
                    vec![(0, 1), (5, 1), (99, 1)]
                );
                assert_eq!(entries[2].status_text, "unknown");
            }
            conn.batch_execute("UPDATE inbox_entries SET scheduled_at = scheduled_at + interval '2 days' WHERE status = 5; UPDATE outbox_entries SET status = 4 WHERE status = 99; INSERT INTO inbox_entries(status, scheduled_at) SELECT 7, scheduled_at FROM inbox_entries WHERE status = 0; INSERT INTO outbox_entries(status, scheduled_at) SELECT 8, scheduled_at FROM outbox_entries WHERE status = 0;").unwrap();
            let changed = load_entry_statuses_at(&mut conn, now).unwrap();
            assert_eq!(
                changed
                    .inbox_entries
                    .iter()
                    .map(|e| (e.status, e.count))
                    .collect::<Vec<_>>(),
                vec![(0, 1), (7, 1), (99, 1)]
            );
            assert_eq!(
                changed
                    .outbox_entries
                    .iter()
                    .map(|e| (e.status, e.count))
                    .collect::<Vec<_>>(),
                vec![(0, 1), (4, 1), (5, 1), (8, 1)]
            );
            assert_eq!(
                changed
                    .command_entries
                    .iter()
                    .map(|e| (e.status, e.count))
                    .collect::<Vec<_>>(),
                vec![(0, 1), (5, 1), (99, 1)]
            );
            conn.batch_execute("UPDATE command_entries SET scheduled_at = scheduled_at + interval '2 days' WHERE status = 5; UPDATE event_entries SET status = 4 WHERE status = 99").unwrap();
            let report = load_entry_statuses_at(&mut conn, now).unwrap();
            assert_eq!(
                report.window_start,
                start.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
            );
            assert_eq!(
                report.window_end,
                end.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
            );
            assert_eq!(
                report.command_entries.iter().map(|e| e.count).sum::<u64>(),
                2
            );
            assert!(
                report
                    .event_entries
                    .iter()
                    .any(|e| e.status == 4 && e.count == 1)
            );
        }
    }
}
