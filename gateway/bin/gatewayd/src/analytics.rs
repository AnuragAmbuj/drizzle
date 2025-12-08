use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

pub struct AnalyticsDb {
    pool: Pool<Postgres>,
}

impl AnalyticsDb {
    pub async fn new(db_url: &str) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(db_url)
            .await?;

        // Initialize schema
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS request_logs (
                id BIGSERIAL PRIMARY KEY,
                timestamp TIMESTAMPTZ NOT NULL,
                method TEXT NOT NULL,
                path TEXT NOT NULL,
                status INTEGER NOT NULL,
                duration_ms DOUBLE PRECISION NOT NULL,
                tenant_id TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_timestamp ON request_logs(timestamp)")
            .execute(&pool)
            .await?;

        Ok(Self { pool })
    }

    pub async fn insert(&self, entry: &proxy::observability::RecentRequest) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO request_logs (timestamp, method, path, status, duration_ms, tenant_id)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(entry.timestamp)
        .bind(&entry.method)
        .bind(&entry.path)
        .bind(entry.status as i32)
        .bind(entry.duration_ms)
        .bind(&entry.tenant_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_stats(&self, start_ts: String) -> anyhow::Result<Vec<(String, i64, i64)>> {
        let start_date =
            chrono::DateTime::parse_from_rfc3339(&start_ts)?.with_timezone(&chrono::Utc);

        // Query aggregated stats
        let rows = sqlx::query_as::<_, (String, i64, i64)>(
            "SELECT to_char(date_trunc('minute', timestamp), 'YYYY-MM-DD\"T\"HH24:MI:00\"Z\"') as time_bucket,
                    CASE 
                        WHEN status >= 500 THEN 500
                        WHEN status = 429 THEN 429
                        WHEN status IN (401, 403) THEN 401
                        WHEN status >= 400 THEN 400
                        ELSE 200
                    END::BIGINT as status_group,
                    COUNT(*) as count
             FROM request_logs
             WHERE timestamp >= $1
             GROUP BY 1, 2
             ORDER BY 1 ASC",
        )
        .bind(start_date)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn get_logs(
        &self,
        limit: i64,
    ) -> anyhow::Result<Vec<proxy::observability::RecentRequest>> {
        let rows = sqlx::query_as::<
            _,
            (
                chrono::DateTime<chrono::Utc>,
                String,
                String,
                i32,
                f64,
                String,
            ),
        >(
            "SELECT timestamp, method, path, status, duration_ms, tenant_id
             FROM request_logs
             ORDER BY timestamp DESC
             LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let results = rows
            .into_iter()
            .map(|(ts, method, path, status, duration, tenant)| {
                proxy::observability::RecentRequest {
                    timestamp: ts,
                    method,
                    path,
                    status: status as u16,
                    duration_ms: duration,
                    tenant_id: tenant,
                }
            })
            .collect();

        Ok(results)
    }
}
