use std::{error::Error, str::FromStr};

use sql_gis::{
    sql_types::SpatiaLitePoint,
    types::{GeometryImpl as _, Point},
};
use sqlx::{sqlite::SqliteConnectOptions, Connection, SqliteConnection};

/// Crée une base de données en mémoire, et charge l'extension SpatiaLite.
async fn connect_to_database() -> Result<SqliteConnection, Box<dyn Error>> {
    let opts = SqliteConnectOptions::from_str(":memory:")?.extension("mod_spatialite");
    let conn = SqliteConnection::connect_with(&opts).await?;
    Ok(conn)
}

/// Crée les tables adéquates pour réaliser l'ensemble des tests d'intégration
async fn setup() -> Result<SqliteConnection, Box<dyn Error>> {
    let mut conn = connect_to_database().await?;

    // Crée une table de points
    let create_points_table =
        sqlx::query("CREATE TABLE gis_points (id INTEGER NOT NULL PRIMARY KEY, pt GEOMETRY)");

    create_points_table.execute(&mut conn).await?;

    Ok(conn)
}

#[sqlx::test]
/// Teste l'encodage/décodage d'une géométrie (point) depuis SpatiaLite
async fn test_spatialite_isomorphism() -> Result<(), Box<dyn Error>> {
    let mut conn = setup().await.expect("cannot setup test environment");

    let expected = SpatiaLitePoint::new([10.1, 20.2]);

    let (id,): (u32,) = sqlx::query_as("INSERT INTO gis_points (pt) VALUES (?) RETURNING id")
        .bind(&expected)
        .fetch_one(&mut conn)
        .await?;

    let (value,): (SpatiaLitePoint,) = sqlx::query_as("SELECT pt FROM gis_points WHERE id = ?")
        .bind(id)
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(expected, value);

    Ok(())
}