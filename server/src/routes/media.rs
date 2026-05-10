use actix_web::{web, HttpResponse, HttpRequest, Responder};
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use std::env;
use crate::db::DbPool;

pub async fn stream_media(
    path: web::Path<String>,
    req: HttpRequest,
    db: web::Data<DbPool>,
) -> impl Responder {
    let audio_id = path.into_inner();

    // Check if the object key exists in the database
    let db = db.lock().unwrap();
    let s3_object_key: Result<String, _> = db.query_row(
        "SELECT s3_object_key FROM audios WHERE id = ?1 AND processing_status = 'ready'",
        rusqlite::params![audio_id],
        |row| row.get(0),
    );

    let s3_key = match s3_object_key {
        Ok(key) => key,
        Err(_) => return HttpResponse::NotFound().body("Media not found or not ready"),
    };

    let s3_endpoint = env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://rustfs:9000".to_string());
    let s3_access_key = env::var("S3_ACCESS_KEY").unwrap_or_else(|_| "rustfsadmin".to_string());
    let s3_secret_key = env::var("S3_SECRET_KEY").unwrap_or_else(|_| "rustfsadmin".to_string());
    let s3_bucket_name = env::var("S3_BUCKET").unwrap_or_else(|_| "blindtest".to_string());
    let s3_region = env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());

    let region = Region::Custom {
        region: s3_region,
        endpoint: s3_endpoint,
    };
    
    let credentials = Credentials::new(
        Some(&s3_access_key),
        Some(&s3_secret_key),
        None,
        None,
        None,
    ).unwrap();

    let bucket = Bucket::new(&s3_bucket_name, region, credentials)
        .unwrap()
        .with_path_style();

    let response_data = bucket.get_object(&s3_key).await;
    match response_data {
        Ok(res) => {
            let bytes = res.bytes().to_vec();
            let total_len = bytes.len();

            // Parse Range header (e.g. "bytes=0-" or "bytes=1024-2047")
            let range_header = req.headers().get("Range").and_then(|v| v.to_str().ok());

            if let Some(range_str) = range_header {
                if let Some(range_val) = range_str.strip_prefix("bytes=") {
                    let parts: Vec<&str> = range_val.splitn(2, '-').collect();
                    if parts.len() == 2 {
                        let start: usize = parts[0].parse().unwrap_or(0);
                        let end: usize = if parts[1].is_empty() {
                            total_len.saturating_sub(1)
                        } else {
                            parts[1].parse().unwrap_or(total_len.saturating_sub(1))
                        };

                        if start < total_len && end < total_len && start <= end {
                            let chunk = bytes[start..=end].to_vec();
                            let chunk_len = chunk.len();
                            return HttpResponse::PartialContent()
                                .content_type("video/mp4")
                                .append_header(("Accept-Ranges", "bytes"))
                                .append_header(("Content-Range", format!("bytes {}-{}/{}", start, end, total_len)))
                                .append_header(("Content-Length", chunk_len.to_string()))
                                .body(chunk);
                        }
                    }
                }
            }

            HttpResponse::Ok()
                .content_type("video/mp4")
                .append_header(("Accept-Ranges", "bytes"))
                .append_header(("Content-Length", total_len.to_string()))
                .body(bytes)
        }
        Err(e) => {
            log::error!("S3 Get Object Error for {}: {:?}", s3_key, e);
            HttpResponse::InternalServerError().body("Failed to retrieve media")
        }
    }
}
