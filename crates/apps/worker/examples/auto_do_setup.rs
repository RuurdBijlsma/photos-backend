#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use app_state::load_app_settings;
use common_services::api::auth::interfaces::CreateUser;
use common_services::api::auth::service::create_user;
use common_services::api::onboarding::service::start_processing;
use common_services::database::get_db_pool;
use common_services::database::user_store::UserStore;
use common_types::dev_constants::{EMAIL, PASSWORD, USERNAME};
use tracing::Level;
use tracing_subscriber::{EnvFilter, fmt};
use worker::worker::create_worker;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| "worker=info,ort=warn".into());
    let subscriber = fmt::Subscriber::builder()
        .with_max_level(Level::INFO)
        .with_env_filter(filter)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    color_eyre::install()?;

    let settings = load_app_settings()?;
    let pool = get_db_pool(&settings.secrets.database_url, true).await?;

    let create_user_payload = CreateUser {
        name: USERNAME.to_owned(),
        email: EMAIL.to_owned(),
        password: PASSWORD.to_owned(),
    };
    let user_result = create_user(&pool, &create_user_payload).await;
    let user = if let Ok(u) = user_result {
        u
    } else {
        UserStore::list_users(&pool)
            .await?
            .first()
            .expect("No dev user found")
            .clone()
    };
    println!("Created user {user:?}");
    let user = start_processing(&pool, &settings, user.id, String::new()).await?;
    println!("Started processing, media folder: {:?}", user.media_folder);
    create_worker(pool, settings, true, true).await?;

    Ok(())
}
