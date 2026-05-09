use std::env;
use std::process::Command;
use std::fs;
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use uuid::Uuid;
use log::{info, error};
use crate::db::DbPool;

pub async fn process_and_upload_video(
    db: actix_web::web::Data<DbPool>,
    audio_id: String,
    video_url: String,
    start_time: i64,
    table: &str, // "audios" or "suggestions"
) {
    let s3_endpoint = env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://rustfs:9000".to_string());
    let s3_access_key = env::var("S3_ACCESS_KEY").unwrap_or_else(|_| "rustfsadmin".to_string());
    let s3_secret_key = env::var("S3_SECRET_KEY").unwrap_or_else(|_| "rustfsadmin".to_string());
    let s3_bucket_name = env::var("S3_BUCKET").unwrap_or_else(|_| "blindtest".to_string());
    let s3_region = env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());

    let region = Region::Custom {
        region: s3_region,
        endpoint: s3_endpoint.clone(),
    };
    
    let credentials = Credentials::new(
        Some(&s3_access_key),
        Some(&s3_secret_key),
        None,
        None,
        None,
    ).unwrap();

    let bucket = Bucket::new(&s3_bucket_name, region.clone(), credentials.clone())
        .unwrap()
        .with_path_style();

    // Create bucket if it doesn't exist
    let _ = Bucket::create_with_path_style(
        &s3_bucket_name,
        region,
        credentials,
        Default::default(),
    ).await;

    let temp_id = Uuid::new_v4().to_string();
    let download_path = format!("/tmp/{}.mp4", temp_id);
    let normalized_path = format!("/tmp/{}_norm.mp4", temp_id);

    let end_time = start_time + 150; // 2.5 minutes clip

    info!("Starting download for video {} from {} to {}", audio_id, start_time, end_time);

    // 1. Download and cut using yt-dlp
    // Use cookies for YouTube auth; Node.js (installed in image) handles the n-challenge
    let cookies_path = "/app/cookies.txt";
    let dl_status = Command::new("yt-dlp")
        .arg("--cookies")
        .arg(cookies_path)
        .arg("-f")
        .arg("bestvideo[height<=720][ext=mp4]+bestaudio[ext=m4a]/best[height<=720][ext=mp4]/best[height<=720]/best")
        .arg("--download-sections")
        .arg(format!("*{}-{}", start_time, end_time))
        .arg("--force-keyframes-at-cuts")
        .arg("--no-playlist")
        .arg("-o")
        .arg(&download_path)
        .arg(&video_url)
        .status();

    let mut path_to_process = download_path.clone();

    if let Ok(status) = dl_status {
        if !status.success() {
            error!("yt-dlp failed for {}. Marking as error.", audio_id);
            set_processing_status(&db, &audio_id, table, "error");
            return;
        }
    } else {
        error!("Failed to execute yt-dlp for {}", audio_id);
        set_processing_status(&db, &audio_id, table, "error");
        return;
    }

    // Check if downloaded
    if !fs::metadata(&download_path).is_ok() {
        error!("Downloaded file not found for {}. yt-dlp may have been blocked by YouTube.", audio_id);
        set_processing_status(&db, &audio_id, table, "error");
        return;
    }

    // 2. Normalize audio with ffmpeg
    info!("Normalizing audio for {}", audio_id);
    let norm_status = Command::new("ffmpeg")
        .arg("-y")
        .arg("-i")
        .arg(&download_path)
        .arg("-af")
        .arg("loudnorm")
        .arg("-c:v")
        .arg("copy")
        .arg(&normalized_path)
        .status();

    if let Ok(status) = norm_status {
        if status.success() && fs::metadata(&normalized_path).is_ok() {
            path_to_process = normalized_path.clone();
        } else {
            error!("ffmpeg normalization failed for {}. Proceeding without normalization.", audio_id);
        }
    } else {
        error!("Failed to execute ffmpeg for {}", audio_id);
    }

    // 3. Upload to S3
    info!("Uploading {} to S3", audio_id);
    let object_key = format!("{}.mp4", audio_id);
    
    if let Ok(file_content) = fs::read(&path_to_process) {
        let response = bucket.put_object_with_content_type(&object_key, &file_content, "video/mp4").await;
        
        match response {
            Ok(res) if res.status_code() == 200 => {
                info!("Upload successful for {}", audio_id);
                // 4. Update Database
                update_db_success(&db, &audio_id, table, &object_key);
            }
            Err(e) => {
                error!("S3 upload error for {}: {:?}", audio_id, e);
                set_processing_status(&db, &audio_id, table, "error");
            }
            Ok(res) => {
                error!("S3 upload failed with status {} for {}", res.status_code(), audio_id);
                set_processing_status(&db, &audio_id, table, "error");
            }
        }
    } else {
        error!("Could not read processed file for {}", audio_id);
        set_processing_status(&db, &audio_id, table, "error");
    }

    // Cleanup
    let _ = fs::remove_file(&download_path);
    let _ = fs::remove_file(&normalized_path);
}

fn set_processing_status(db: &actix_web::web::Data<DbPool>, id: &str, table: &str, status: &str) {
    let db = db.lock().unwrap();
    let query = format!("UPDATE {} SET processing_status = ?1 WHERE id = ?2", table);
    let _ = db.execute(&query, rusqlite::params![status, id]);
}

fn update_db_success(db: &actix_web::web::Data<DbPool>, id: &str, table: &str, s3_key: &str) {
    let db = db.lock().unwrap();
    let query = format!("UPDATE {} SET processing_status = 'ready', s3_object_key = ?1 WHERE id = ?2", table);
    let _ = db.execute(&query, rusqlite::params![s3_key, id]);
}
