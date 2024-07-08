use std::{
    error::Error,
    ops::{Deref, DerefMut},
};

use sql_gis::{sql_types::PgPoint, types::{GeometryImpl as _, Point}};
use sqlx::{postgres::PgConnectOptions, Connection, PgConnection};

struct PgInstance {
    conn: PgConnection,
}

impl Deref for PgInstance {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

impl DerefMut for PgInstance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.conn
    }
}

async fn connect_to_database() -> Result<PgInstance, Box<dyn Error>> {
    let opts = PgConnectOptions::new()
        .database("postgres")
        .username("postgres")
        .password("postgres")
        .host("localhost")
        .ssl_mode(sqlx::postgres::PgSslMode::Disable)
        .port(5432);

    let conn = PgConnection::connect_with(&opts)
        .await
        .expect(&format!("failed to connect to database",));

    Ok(PgInstance { conn })
}

/// Crée les tables adéquates pour réaliser l'ensemble des tests d'intégration
async fn setup() -> Result<PgInstance, Box<dyn Error>> {
    let mut instance = connect_to_database().await?;

    sqlx::query("BEGIN").execute(instance.deref_mut()).await?;

    // Crée une table de points
    let create_points_table =
        sqlx::query("CREATE TABLE gis_points (id SERIAL PRIMARY KEY, pt GEOMETRY)");

    create_points_table.execute(instance.deref_mut()).await?;

    Ok(instance)
}

#[sqlx::test]
/// Teste l'encodage/décodage d'une géométrie (point) depuis la BDD
async fn test_postgis_isomorphism() -> Result<(), Box<dyn Error>> {
    let mut instance = setup().await.expect("failed to setup environment");

    let expected = PgPoint::new([10.1, 20.2]);

    let (id,): (i32,) = sqlx::query_as("INSERT INTO gis_points (pt) VALUES ($1) RETURNING id")
        .bind(&expected)
        .fetch_one(instance.deref_mut())
        .await
        .expect("failed to insert geometry");

    let (value,): (PgPoint,) = sqlx::query_as("SELECT pt FROM gis_points WHERE id = $1")
        .bind(id)
        .fetch_one(instance.deref_mut())
        .await
        .expect("failed to retrieve geometry");

    assert_eq!(expected, value);

    Ok(())
}
