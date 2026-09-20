use utoipa::OpenApi;

use crate::dto::CreateLocationRequest;
use geo_domain::{Location, LocationPrecision};
use geo_geocoding::{AdministrativeArea, AreaBreadcrumb, ReverseGeocodeResult};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::health::health_check,
        crate::handlers::locations::create_location,
        crate::handlers::geocoding::reverse_geocode_handler,
        crate::handlers::geocoding::list_admin_areas_handler,
        crate::handlers::map::search_viewport,
        crate::handlers::map::search_nearby,
        crate::handlers::tiles::serve_base_map_tile,
        crate::handlers::tiles::serve_vector_tile
    ),
    components(
        schemas(
            CreateLocationRequest,
            Location,
            LocationPrecision,
            ReverseGeocodeResult,
            AdministrativeArea,
            AreaBreadcrumb
        )
    ),
    tags(
        (name = "Health", description = "Service health endpoints"),
        (name = "Locations", description = "Location point management"),
        (name = "Geocoding", description = "Reverse geocoding and administrative areas"),
        (name = "Map", description = "Map viewport features and proximity querying"),
        (name = "Tiles", description = "Vector tile generation (MVT/Protobuf)")
    )
)]
pub struct MapApiDoc;