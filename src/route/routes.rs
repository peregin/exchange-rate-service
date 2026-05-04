use crate::route::api;
use crate::route::index;

use actix_web::web;

pub fn init_routes(config: &mut web::ServiceConfig) {
    index::init_routes(config);
    api::init_routes(config);
    config.service(actix_files::Files::new("/static", "static"));
}
