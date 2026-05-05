use actix_web::{web, HttpResponse, HttpRequest, Responder};
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use std::env;
use crate::db::DbPool;

pub async fn stream_media(
    path: web::Path<String>,
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
            // For a robust implementation, we should stream it, but returning the whole body is okay for small clips.
            // A 2.5 min MP4 is a few MBs.
            let bytes = res.bytes().to_vec();
            HttpResponse::Ok()
                .content_type("video/mp4")
                .append_header(("Accept-Ranges", "bytes"))
                .body(bytes)
        }
        Err(e) => {
            log::error!("S3 Get Object Error for {}: {:?}", s3_key, e);
            HttpResponse::InternalServerError().body("Failed to retrieve media")
        }
    }
}
