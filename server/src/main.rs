mod db;
mod middleware;
mod models;
mod routes;
mod ws;
mod video_processor;

use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    log::info!("Initializing database...");
    let db_pool = db::init_db().expect("Failed to initialize database");
    log::info!("Database initialized.");

    let processing_queue = video_processor::start_queue_worker();
    log::info!("Processing queue worker started.");

    let auth_state = middleware::AuthState::new();
    let broadcaster = ws::WsBroadcaster::new();

    let port = std::env::var("PORT").unwrap_or_else(|_| "80".to_string());
    let bind_addr = format!("0.0.0.0:{}", port);
    log::info!("Starting server on {}", bind_addr);

    HttpServer::new(move || {
        let mut cors = Cors::default()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        if let Ok(urls) = std::env::var("FRONTEND_URL") {
            for url in urls.split(',') {
                cors = cors.allowed_origin(url.trim());
            }
        } else {
            cors = cors.allow_any_origin();
        }

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(auth_state.clone()))
            .app_data(web::Data::new(broadcaster.clone()))
            .app_data(web::Data::new(processing_queue.clone()))
            // Auth
            .route("/signin", web::post().to(routes::auth::signin))
            .route("/signup", web::post().to(routes::auth::signup))
            .route("/confirmemail", web::post().to(routes::auth::confirm_email))
            .route("/resetpassword", web::post().to(routes::auth::reset_password))
            .route("/changepassword", web::post().to(routes::auth::change_password))
            // Users
            .route("/getuser", web::get().to(routes::users::get_user))
            .route("/getusers", web::get().to(routes::users::get_users))
            .route("/getcontributorusers", web::get().to(routes::users::get_contributor_users))
            .route("/updateprofile", web::post().to(routes::users::update_profile))
            .route("/updateuser", web::post().to(routes::users::update_user))
            .route("/deleteuser", web::delete().to(routes::users::delete_user))
            .route("/getUserProfile", web::post().to(routes::users::get_user_profile))
            // Audios
            .route("/getnextaudio", web::get().to(routes::audios::get_next_audio))
            .route("/newaudio", web::post().to(routes::audios::new_audio))
            .route("/suggestaudio", web::post().to(routes::audios::suggest_audio))
            .route("/getallaudios", web::get().to(routes::audios::get_all_audios))
            .route("/updateaudio", web::post().to(routes::audios::update_audio))
            .route("/flagaudio", web::post().to(routes::audios::flag_audio))
            .route("/resetflag", web::post().to(routes::audios::reset_flag))
            .route("/deleteaudio", web::delete().to(routes::audios::delete_audio))
            .route("/backupaudio", web::get().to(routes::audios::backup_audios))
            .route("/testanswer", web::post().to(routes::audios::test_answer))
            // Custom blindtests
            .route("/createcustomblindtest", web::post().to(routes::custom_blindtests::create))
            .route("/getcustomblindtests", web::get().to(routes::custom_blindtests::get_user_blindtests))
            .route("/getpubliccustomblindtests", web::get().to(routes::custom_blindtests::get_public_blindtests))
            .route("/getcustomblindtest/{id}", web::get().to(routes::custom_blindtests::get_one))
            .route("/updatecustomblindtest/{id}", web::post().to(routes::custom_blindtests::update))
            .route("/deletecustomblindtest/{id}", web::delete().to(routes::custom_blindtests::delete))
            .route("/getaudiosnames", web::get().to(routes::custom_blindtests::get_audio_names))
            // Canvas
            .route("/getCanvas", web::get().to(routes::canvas::get_canvas))
            .route("/getPixelData", web::get().to(routes::canvas::get_pixel_data))
            .route("/updatePixel", web::post().to(routes::canvas::update_pixel))
            // Suggestions
            .route("/getSuggestions", web::get().to(routes::suggestions::get_suggestions))
            // Media
            .route("/media/{id}", web::get().to(routes::media::stream_media))
            // Stats
            .route("/getBlindtestStats", web::get().to(routes::stats::get_blindtest_stats))
            .route("/getCanvasStats", web::get().to(routes::stats::get_canvas_stats))
            // WebSocket
            .route("/ws", web::get().to(ws::ws_handler))
    })
    .bind(&bind_addr)?
    .run()
    .await
}
