use sea_orm::Iden;
use sea_query::{Func, FunctionCall, SimpleExpr};

/// Retourne la distance entre deux géométries
pub fn st_distance<G1, G2>(geom1: G1, geom2: G2) -> FunctionCall
where
    G1: Into<SimpleExpr>,
    G2: Into<SimpleExpr>,
{
    Func::cust(StDistance).arg(geom1).arg(geom2)
}

/// Vérifie si geom1 est complètement dans geom2
pub fn st_within<G1, G2>(geom1: G1, geom2: G2) -> FunctionCall
where
    G1: Into<SimpleExpr>,
    G2: Into<SimpleExpr>,
{
    Func::cust(StWithin).arg(geom1).arg(geom2)
}

#[derive(Iden)]
#[iden = "ST_Distance"]
struct StDistance;

#[derive(Iden)]
#[iden = "ST_Within"]
struct StWithin;
