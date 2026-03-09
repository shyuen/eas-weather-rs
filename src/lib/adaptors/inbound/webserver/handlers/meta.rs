use crate::adaptors::inbound::webserver::poem::AppState;
use crate::adaptors::inbound::webserver::poem::OperationalTags;
use crate::core::domain::meta::ports::Meta;

use poem::web::Data;
use poem_openapi::{ApiResponse, OpenApi, payload::Json};
use serde_json::Value;

pub struct MetaHandler;

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
impl MetaHandler {
    /// Outputs application configuration information
    #[oai(path = "/info", method = "get", hidden = false)]
    async fn info(
        &self,
        //Data(app_state): Data<&AppState<M>>
    ) -> MetaResponses
where
        //M: Meta,
    {
        // Return the configuration with app_state as JSON

        //app_state.meta.get_app_data().await;

        MetaResponses::Ok(Json(serde_json::json!(
            // Access the inner Config reference
            {
                "app_data": "test",
            }
        )))
    }
}

async fn info<M>(Data(_app_state): Data<&AppState<M>>) -> MetaResponses
where
    M: Meta,
{
    // Return the configuration with app_state as JSON

    //app_state.meta.get_app_data().await;

    MetaResponses::Ok(Json(serde_json::json!(
        // Access the inner Config reference
        {
            "app_data": "test",
        }
    )))
}
