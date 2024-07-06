use crate::ewkb;

pub type PgGeometry = ewkb::EWKBGeometry;

pub type PgPoint = ewkb::EWKBPoint;
pub type PgLineString = ewkb::EWKBLineString;
pub type PgPolygon = ewkb::EWKBPolygon;
pub type PgMultiPoint = ewkb::EWKBMultiPoint;
pub type PgMultiLineString = ewkb::EWKBMultiLineString;
pub type PgMultiPolygon = ewkb::EWKBMultiPolygon;

pub type PgPointZ = ewkb::EWKBPointZ;
pub type PgLineStringZ = ewkb::EWKBLineStringZ;
pub type PgPolygonZ = ewkb::EWKBPolygonZ;
pub type PgMultiPointZ = ewkb::EWKBMultiPointZ;
pub type PgMultiLineStringZ = ewkb::EWKBMultiLineStringZ;
pub type PgMultiPolygonZ = ewkb::EWKBMultiPolygonZ;
