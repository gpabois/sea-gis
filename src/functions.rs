use sea_orm::Iden;
use sea_query::{Func, FunctionCall, IntoIden};

/// Retourne la distance entre deux géométries
pub fn st_distance<G1, G2>(geom1: G1, geom2: G2) -> FunctionCall 
where G1: IntoIden, G2: IntoIden 
{
    Func::cust(StDistance).args((
        geom1,
        geom2
    ))
}

/// Vérifie si geom1 est complètement dans geom2
pub fn st_within<G1, G2>(geom1: G1, geom2: G2) -> FunctionCall 
where G1: IntoIden, G2: IntoIden 
{
    Func::cust(StWithin).args((
        geom1,
        geom2
    ))
}


#[derive(Iden)]
#[iden = "ST_Distance"]
pub struct StDistance;


#[derive(Iden)]
#[iden = "ST_Within"]
pub struct StWithin;

