use crate::models::{Data, EnvConfig, OptionalData, RowId};
use anyhow::{anyhow, Result};
use deadpool_postgres::{Config, Pool, Runtime, SslMode};
use rustls::Certificate;
use std::path::Path;
use tokio_postgres::types::Type;

#[derive(Clone)]
pub(crate) struct Database(Pool);

impl Database {
    pub(crate) async fn connect(env_config: &EnvConfig) -> Result<Database> {
        // Connect to the database.
        let mut config = Config::new();
        config.host = Some(env_config.db_host.clone());
        config.port = Some(env_config.db_port);
        config.dbname = Some(env_config.db_name.clone());
        config.user = Some(env_config.db_user.clone());
        config.password = Some(env_config.db_password.clone());
        config.ssl_mode = Some(SslMode::Require);

        let mut certs = rustls::RootCertStore::empty();
        certs.add(&Certificate(base64::decode(&env_config.db_cert)?))?;

        let tls_config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(certs)
            .with_no_client_auth();
        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(tls_config);

        let pool = config.create_pool(Some(Runtime::Tokio1), tls)?;
        // Verify sucessful connection
        let _ = pool.get().await?;

        Ok(Database(pool))
    }

    pub(crate) async fn get_datapoint(&mut self) -> Result<(Data, RowId)> {
        let conn = self.0.get().await?;

        let stmt = conn
            .prepare_cached(
                r#"
        WITH row AS (
            SELECT id
            FROM simulations
            WHERE complete = false
            ORDER BY run_order
            LIMIT 1
        )
        UPDATE simulations s
        SET run_order = nextval('run_order_seq')
        FROM row
        WHERE s.id = row.id
        RETURNING tip_b, root_b, span_b, sweep_b, body_length_b, 
                  tip_s, root_s, span_s, sweep_s, body_length_s, 
                  body_diameter_bs, mach_number, s.id;
        "#,
            )
            .await?;

        let row = conn.query_one(&stmt, &[]).await?;

        Ok((
            Data {
                tip_b: row.get(0),
                root_b: row.get(1),
                span_b: row.get(2),
                sweep_b: row.get(3),
                body_length_b: row.get(4),
                tip_s: row.get(5),
                root_s: row.get(6),
                span_s: row.get(7),
                sweep_s: row.get(8),
                body_length_s: row.get(9),
                body_diameter_bs: row.get(10),
                mach_number: row.get(11),
                power_on_bs: None,
                power_off_bs: None,
                power_on_s: None,
                power_off_s: None,
            },
            RowId(row.get(12)),
        ))
    }

    pub(crate) async fn set_datapoint(&mut self, row: RowId, data: OptionalData) -> Result<()> {
        let conn = self.0.get().await?;

        let stmt = conn
            .prepare_typed_cached(
                r#"UPDATE simulations
                     SET power_on_bs = $2, power_off_bs = $3, 
                     power_on_s = $4, power_off_s = $5, complete = true
                     WHERE id = $1 AND complete = false;"#,
                &[
                    Type::INT8,
                    Type::FLOAT8,
                    Type::FLOAT8,
                    Type::FLOAT8,
                    Type::FLOAT8,
                ],
            )
            .await?;

        let rows = conn
            .execute(
                &stmt,
                &[
                    &row.0,
                    &data.power_on_bs,
                    &data.power_off_bs,
                    &data.power_on_s,
                    &data.power_off_s,
                ],
            )
            .await?;

        if rows == 0 {
            Err(anyhow!("Task does not exist or was already submitted!"))
        } else {
            Ok(())
        }
    }

    pub(crate) async fn setup(&mut self, path: &Path) -> Result<()> {
        // Load in file
        let mut reader = csv::Reader::from_path(path)?;
        let records: Result<Vec<Data>, csv::Error> = reader.deserialize().collect();
        let records = records?;

        // Setup table and sequence
        let conn = self.0.get().await?;
        conn.execute(
            r#"
        CREATE TABLE simulations
        (
            id               serial PRIMARY KEY,
            tip_b            double precision NOT NULL,
            root_b           double precision NOT NULL,
            span_b           double precision NOT NULL,
            sweep_b          double precision NOT NULL,
            body_length_b    double precision NOT NULL,
            tip_s            double precision NOT NULL,
            root_s           double precision NOT NULL,
            span_s           double precision NOT NULL,
            sweep_s          double precision NOT NULL,
            body_length_s    double precision NOT NULL,
            body_diameter_bs double precision NOT NULL,
            mach_number      double precision NOT NULL,
            power_on_bs      double precision,
            power_off_bs     double precision,
            power_on_s       double precision,
            power_off_s      double precision,
            complete         boolean          NOT NULL,
            run_order        integer          NOT NULL
        );"#,
            &[],
        )
        .await?;

        conn.execute("CREATE SEQUENCE run_order_seq;", &[]).await?;

        // Insert data
        for record_chunk in records.chunks(20) {
            self.insert_simulation_rows(record_chunk).await?;
        }

        // Setup index
        conn.execute(
            "CREATE INDEX run_order_index ON simulations (run_order);
        ",
            &[],
        )
        .await?;

        Ok(())
    }

    async fn insert_simulation_rows(&mut self, data_slice: &[Data]) -> Result<()> {
        let mut conn = self.0.get().await?;
        let tx = conn.transaction().await?;

        let stmt = tx
            .prepare_cached(
                r#"
                INSERT INTO simulations 
                (tip_b, root_b, span_b, sweep_b, body_length_b,
                tip_s, root_s, span_s, sweep_s, body_length_s,
                body_diameter_bs, mach_number, complete, run_order)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 
                    false, nextval('run_order_seq'));"#,
            )
            .await?;

        for data in data_slice {
            tx.execute(
                &stmt,
                &[
                    &data.tip_b,
                    &data.root_b,
                    &data.span_b,
                    &data.sweep_b,
                    &data.body_length_b,
                    &data.tip_s,
                    &data.root_s,
                    &data.span_s,
                    &data.sweep_s,
                    &data.body_length_s,
                    &data.body_diameter_bs,
                    &data.mach_number,
                ],
            )
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }
}
