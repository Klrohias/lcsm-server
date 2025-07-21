use axum::Router;

#[allow(unused_imports)]
use log::info;

use crate::AppStateRef;

#[cfg(feature = "docker")]
mod docker;
mod instances;
mod processes;

#[allow(unused_mut)]
pub fn get_routes(state_ref: &AppStateRef) -> Router {
    let mut result = Router::new()
        .nest("/instances", instances::get_routes(state_ref))
        .nest("/processes", processes::get_routes(state_ref));

    #[cfg(feature = "docker")]
    {
        result = result.nest("/docker", docker::get_routes(state_ref));
        info!("Docker support is enabled");
    }

    result
}
