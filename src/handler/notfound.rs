use actix_web::{HttpResponse,HttpRequest};
pub async fn not_found_handler(req: HttpRequest) -> HttpResponse {
    HttpResponse::NotFound().json(serde_json::json!({
        "error": "Route not found",
        "path": req.path(),
        "method": req.method().as_str()
    }))
}