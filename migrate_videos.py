import sqlite3
import os
import subprocess
import boto3
import tempfile
import uuid

# Configuration
DB_PATH = "server/data/blindtest.db"
S3_ENDPOINT = os.environ.get("S3_ENDPOINT", "http://localhost:9000")
S3_ACCESS_KEY = os.environ.get("S3_ACCESS_KEY", "rustfsadmin")
S3_SECRET_KEY = os.environ.get("S3_SECRET_KEY", "rustfsadmin")
S3_BUCKET = os.environ.get("S3_BUCKET", "blindtest")
S3_REGION = os.environ.get("S3_REGION", "us-east-1")

s3 = boto3.client(
    's3',
    endpoint_url=S3_ENDPOINT,
    aws_access_key_id=S3_ACCESS_KEY,
    aws_secret_access_key=S3_SECRET_KEY,
    region_name=S3_REGION
)

# Ensure bucket exists
try:
    s3.head_bucket(Bucket=S3_BUCKET)
except Exception:
    try:
        s3.create_bucket(Bucket=S3_BUCKET)
    except Exception as e:
        print(f"Could not create bucket: {e}")

def process_video(conn, table_name, row):
    video_id, video_url, start_time, processing_status, s3_key = row

    if processing_status == 'ready' and s3_key:
        print(f"Skipping {video_id}, already processed.")
        return

    print(f"Processing {video_id} from {table_name}...")
    start_time = start_time or 0
    end_time = start_time + 150  # 2.5 mins
    temp_id = str(uuid.uuid4())
    dl_path = f"{tempfile.gettempdir()}/{temp_id}.mp4"
    norm_path = f"{tempfile.gettempdir()}/{temp_id}_norm.mp4"

    # Reset flagged state: not flagged by default
    conn.execute("DELETE FROM flagged_audios WHERE audio_id = ?", (video_id,))
    conn.commit()

    try:
        # 1. Download at 720p
        dl_cmd = [
            "yt-dlp",
            "--cookies", "cookies.txt",
            "-f", "bestvideo[height<=720][ext=mp4]+bestaudio[ext=m4a]/best[height<=720][ext=mp4]/best[height<=720]",
            "--download-sections", f"*{start_time}-{end_time}",
            "--force-keyframes-at-cuts",
            "-o", dl_path,
            video_url
        ]
        res = subprocess.run(dl_cmd, capture_output=True, text=True)
        if res.returncode != 0 or not os.path.exists(dl_path):
            print(f"Failed to download {video_id}: {res.stderr}")
            conn.execute(f"UPDATE {table_name} SET processing_status = 'error' WHERE id = ?", (video_id,))
            # Flag the audio since the original video could not be downloaded
            conn.execute(
                "INSERT OR IGNORE INTO flagged_audios (id, audio_id, user_id, report_message, date) VALUES (?, ?, ?, ?, datetime('now'))",
                (f"{video_id}-migration", video_id, "migration", "Original video failed to download")
            )
            conn.commit()
            return

        # 2. Normalize audio
        norm_cmd = [
            "ffmpeg", "-y", "-i", dl_path,
            "-af", "loudnorm",
            "-c:v", "copy",
            norm_path
        ]
        res = subprocess.run(norm_cmd, capture_output=True, text=True)
        path_to_upload = norm_path if (res.returncode == 0 and os.path.exists(norm_path)) else dl_path

        # 3. Upload to S3
        object_key = f"{video_id}.mp4"
        s3.upload_file(path_to_upload, S3_BUCKET, object_key, ExtraArgs={'ContentType': 'video/mp4'})

        # 4. Update DB
        conn.execute(f"UPDATE {table_name} SET processing_status = 'ready', s3_object_key = ? WHERE id = ?", (object_key, video_id))
        conn.commit()
        print(f"Successfully processed {video_id}.")

    except Exception as e:
        print(f"Exception processing {video_id}: {e}")
        conn.execute(f"UPDATE {table_name} SET processing_status = 'error' WHERE id = ?", (video_id,))
        conn.commit()
    finally:
        if os.path.exists(dl_path): os.remove(dl_path)
        if os.path.exists(norm_path): os.remove(norm_path)

def main():
    if not os.path.exists(DB_PATH):
        print(f"Database not found at {DB_PATH}")
        return

    conn = sqlite3.connect(DB_PATH)
    tables = ["audios", "suggestions"]

    for table in tables:
        try:
            cursor = conn.execute(f"SELECT id, video_url, start_time, processing_status, s3_object_key FROM {table}")
            rows = cursor.fetchall()
            for row in rows:
                process_video(conn, table, row)
        except Exception as e:
            print(f"Error querying {table}: {e}")

    conn.close()
    print("Video migration complete.")

if __name__ == "__main__":
    main()
