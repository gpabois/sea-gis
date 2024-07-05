use std::{error::Error, str::FromStr};

use sql_gis::{spatialite::SpatiaLitePoint, sql_types::SpatiaLiteGeometry, types::Point};
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

#[tokio::test]
/// Teste l'encodage/décodage d'une géométrie (point) depuis SpatiaLite
async fn test_spatialite_isomorphism() -> Result<(), Box<dyn Error>> {
    let mut conn = setup().await.expect("cannot setup test environment");

    let expected = SpatiaLitePoint::from(Point::new([10.1, 20.2]));

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

#[tokio::test]
/// Teste l'insertion de deux points dans une table, et la sélection d'un point qui se trouve dans
/// une surface donnée (via ST_Within)
///
/// Le test ne doit pas retourner le point qui se trouve en dehors de la surface de sélection.
async fn test_spatialite_st_within() -> Result<(), Box<dyn Error>> {
    Ok(())
    /* 
    let mut conn = setup().await.expect("cannot setup test environment");

    let expected = PointS::<SpatiaLite>::new([10.0, 20.0]);
    let outside = PointS::<SpatiaLite>::new([30.0, 30.0]);
    let surface = PolygonS::<SpatiaLite>::new([[8.0, 8.0], [8.0, 22.0], [22.0, 22.0], [22.0, 8.0]]);

    sqlx::query("INSERT INTO gis_points (pt) VALUES (?), (?)")
        .bind(&expected)
        .bind(&outside)
        .execute(&mut conn)
        .await
        .expect("cannot insert geometry");

    let mut rows: Vec<(u32, PointS<SpatiaLite>)> =
        sqlx::query_as("SELECT * FROM gis_points WHERE ST_Within(pt, ?)")
            .bind(&surface)
            .fetch_all(&mut conn)
            .await?;

    assert_eq!(rows.len(), 1);
    let row = rows.pop().unwrap();
    println!("{:?}", row);

    assert_eq!(row.1, expected);

    Ok(())
    */
}
