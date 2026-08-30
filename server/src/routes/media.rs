use actix_web::{web, HttpResponse, HttpRequest, Responder};
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use std::env;
use crate::db::{lock_db, DbPool};

/// Largest slice served for a single open-ended range request (e.g. `bytes=0-`).
/// Ranges are buffered in memory before being sent, so without a cap every player
/// starting a track pulls a whole file into the server's RAM at once. Clients just
/// ask for the next slice when they need it.
const MAX_RANGE_CHUNK: u64 = 4 * 1024 * 1024;

/// A parsed, satisfiable byte range (inclusive on both ends).
#[derive(Debug, PartialEq)]
enum RangeSpec {
    /// No `Range` header: send the whole object.
    Full,
    Partial { start: u64, end: u64 },
    /// The header was present but cannot be satisfied for this object.
    Unsatisfiable,
}

/// Parse a single-range `Range: bytes=…` header against a known object size.
/// Multi-range requests are not supported and fall back to the full object.
fn parse_range(header: Option<&str>, total_len: u64) -> RangeSpec {
    let Some(header) = header else { return RangeSpec::Full };
    let Some(spec) = header.trim().strip_prefix("bytes=") else { return RangeSpec::Full };
    if spec.contains(',') || total_len == 0 {
        return RangeSpec::Full;
    }

    let Some((raw_start, raw_end)) = spec.split_once('-') else { return RangeSpec::Full };
    let (raw_start, raw_end) = (raw_start.trim(), raw_end.trim());

    let (start, end) = if raw_start.is_empty() {
        // Suffix form: `bytes=-500` means the last 500 bytes.
        match raw_end.parse::<u64>() {
            Ok(0) | Err(_) => return RangeSpec::Unsatisfiable,
            Ok(suffix) => (total_len.saturating_sub(suffix), total_len - 1),
        }
    } else {
        let Ok(start) = raw_start.parse::<u64>() else { return RangeSpec::Unsatisfiable };
        let end = if raw_end.is_empty() {
            total_len - 1
        } else {
            match raw_end.parse::<u64>() {
                Ok(end) => end.min(total_len - 1),
                Err(_) => return RangeSpec::Unsatisfiable,
            }
        };
        (start, end)
    };

    if start > end || start >= total_len {
        return RangeSpec::Unsatisfiable;
    }

    // Serving fewer bytes than asked for is allowed; the client requests the rest.
    let end = end.min(start + MAX_RANGE_CHUNK - 1);
    RangeSpec::Partial { start, end }
}

fn build_bucket() -> Result<Box<Bucket>, String> {
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
    ).map_err(|e| format!("invalid S3 credentials: {:?}", e))?;

    Bucket::new(&s3_bucket_name, region, credentials)
        .map(|b| b.with_path_style())
        .map_err(|e| format!("could not build S3 bucket: {:?}", e))
}

pub async fn stream_media(
    path: web::Path<String>,
    req: HttpRequest,
    db: web::Data<DbPool>,
) -> impl Responder {
    let audio_id = path.into_inner();

    // Check if the object key exists in the database.
    // The guard is scoped so it is released before the S3 awaits below: holding it
    // across an await deadlocks the actix worker thread that polls this future.
    let s3_object_key: Result<String, _> = {
        let db = lock_db(&db);
        db.query_row(
            "SELECT s3_object_key FROM audios WHERE id = ?1 AND processing_status = 'ready'",
            rusqlite::params![audio_id],
            |row| row.get(0),
        )
    };

    let s3_key = match s3_object_key {
        Ok(key) => key,
        Err(_) => return HttpResponse::NotFound().body("Media not found or not ready"),
    };

    let bucket = match build_bucket() {
        Ok(bucket) => bucket,
        Err(e) => {
            log::error!("S3 configuration error: {}", e);
            return HttpResponse::InternalServerError().body("Failed to retrieve media");
        }
    };

    // Object size, needed for Content-Range and for clamping the requested range.
    // HEAD does not transfer the body.
    let total_len = match bucket.head_object(&s3_key).await {
        Ok((head, status)) if (200..300).contains(&status) => {
            head.content_length.unwrap_or(0).max(0) as u64
        }
        Ok((_, status)) => {
            log::error!("S3 HEAD returned status {} for {}", status, s3_key);
            return HttpResponse::NotFound().body("Media not found");
        }
        Err(e) => {
            log::error!("S3 Head Object Error for {}: {:?}", s3_key, e);
            return HttpResponse::InternalServerError().body("Failed to retrieve media");
        }
    };

    let range_header = req.headers().get("Range").and_then(|v| v.to_str().ok());

    match parse_range(range_header, total_len) {
        RangeSpec::Unsatisfiable => HttpResponse::RangeNotSatisfiable()
            .append_header(("Content-Range", format!("bytes */{}", total_len)))
            .finish(),

        RangeSpec::Partial { start, end } => {
            match bucket.get_object_range(&s3_key, start, Some(end)).await {
                Ok(res) => {
                    let mut bytes = res.bytes().to_vec();
                    // Defensive: if the store ignored the Range header and sent the whole
                    // object, slice it here rather than mislabelling the response.
                    if res.status_code() == 200 && bytes.len() as u64 > end - start + 1 {
                        let from = (start as usize).min(bytes.len());
                        let to = (end as usize + 1).min(bytes.len());
                        bytes = bytes[from..to].to_vec();
                    }
                    let chunk_len = bytes.len();
                    HttpResponse::PartialContent()
                        .content_type("video/mp4")
                        .append_header(("Accept-Ranges", "bytes"))
                        .append_header(("Content-Range", format!("bytes {}-{}/{}", start, start + chunk_len.saturating_sub(1) as u64, total_len)))
                        .body(bytes)
                }
                Err(e) => {
                    log::error!("S3 Get Object Range Error for {}: {:?}", s3_key, e);
                    HttpResponse::InternalServerError().body("Failed to retrieve media")
                }
            }
        }

        // Whole object: streamed straight through, so memory stays flat no matter
        // how large the file or how many players are fetching at once.
        RangeSpec::Full => match bucket.get_object_stream(&s3_key).await {
            Ok(res) => HttpResponse::Ok()
                .content_type("video/mp4")
                .append_header(("Accept-Ranges", "bytes"))
                .streaming(res.bytes),
            Err(e) => {
                log::error!("S3 Get Object Error for {}: {:?}", s3_key, e);
                HttpResponse::InternalServerError().body("Failed to retrieve media")
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KB: u64 = 1024;

    #[test]
    fn no_header_or_unparsable_unit_serves_whole_object() {
        assert_eq!(parse_range(None, 100), RangeSpec::Full);
        assert_eq!(parse_range(Some("items=0-10"), 100), RangeSpec::Full);
        // Multi-range is not supported; fall back to the full object.
        assert_eq!(parse_range(Some("bytes=0-9,20-29"), 100), RangeSpec::Full);
        // An empty object has no satisfiable range.
        assert_eq!(parse_range(Some("bytes=0-"), 0), RangeSpec::Full);
    }

    #[test]
    fn open_ended_range_runs_to_the_end_of_a_small_object() {
        assert_eq!(parse_range(Some("bytes=0-"), 100), RangeSpec::Partial { start: 0, end: 99 });
        assert_eq!(parse_range(Some("bytes=40-"), 100), RangeSpec::Partial { start: 40, end: 99 });
    }

    #[test]
    fn open_ended_range_is_capped_on_a_large_object() {
        // What browsers actually send when opening a media element.
        assert_eq!(
            parse_range(Some("bytes=0-"), 64 * 1024 * KB),
            RangeSpec::Partial { start: 0, end: MAX_RANGE_CHUNK - 1 },
        );
    }

    #[test]
    fn explicit_range_is_honoured_and_clamped_to_the_object() {
        assert_eq!(parse_range(Some("bytes=10-19"), 100), RangeSpec::Partial { start: 10, end: 19 });
        assert_eq!(parse_range(Some("bytes=90-500"), 100), RangeSpec::Partial { start: 90, end: 99 });
        assert_eq!(parse_range(Some("bytes=99-99"), 100), RangeSpec::Partial { start: 99, end: 99 });
    }

    #[test]
    fn suffix_range_counts_back_from_the_end() {
        assert_eq!(parse_range(Some("bytes=-20"), 100), RangeSpec::Partial { start: 80, end: 99 });
        // Longer than the object: clamped to the whole object.
        assert_eq!(parse_range(Some("bytes=-500"), 100), RangeSpec::Partial { start: 0, end: 99 });
    }

    #[test]
    fn out_of_bounds_and_malformed_ranges_are_rejected() {
        assert_eq!(parse_range(Some("bytes=100-"), 100), RangeSpec::Unsatisfiable);
        assert_eq!(parse_range(Some("bytes=200-300"), 100), RangeSpec::Unsatisfiable);
        assert_eq!(parse_range(Some("bytes=50-10"), 100), RangeSpec::Unsatisfiable);
        assert_eq!(parse_range(Some("bytes=-0"), 100), RangeSpec::Unsatisfiable);
        assert_eq!(parse_range(Some("bytes=abc-def"), 100), RangeSpec::Unsatisfiable);
    }
}
