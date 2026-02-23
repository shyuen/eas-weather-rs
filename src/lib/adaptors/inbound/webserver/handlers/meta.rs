use crate::adaptors::inbound::webserver::poem::OperationalTags;
use crate::core::ports::inbound::config::ConfigRepo;
use crate::core::services::meta::MetaService;

use poem::web::Data;
use poem_openapi::{ApiResponse, OpenApi, payload::Json};
use serde_json::Value;

pub struct Meta;

/// API responses for Meta endpoints
#[derive(ApiResponse)]
enum MetaResponses {
    /// Returns information on application configuration in JSON.
    #[oai(status = 200)]
    Ok(Json<Value>),
    // #[oai(status = 500)]
    // InternalServerError(PlainText<String>),
}

#[OpenApi(prefix_path = "/meta", tag = "OperationalTags::Meta")]
impl Meta {
    /// Outputs application configuration information
    #[oai(path = "/info", method = "get", hidden = false)]
    async fn info(
        &self,
        //meta_serv: Data<&MetaService<C>>
    ) -> MetaResponses
where
        //C: ConfigRepo,
    {
        // Return the configuration with app_state as JSON
        MetaResponses::Ok(Json(serde_json::json!(
            // Access the inner Config reference
            //meta_serv.get_app_config()
            {}
        )))
    }
}
