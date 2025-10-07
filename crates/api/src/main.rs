use axum::{
    http::StatusCode, routing::{get, post},
    Json,
    Router,
};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user));

    // run our app with hyper, listening globally on port 3567
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3567").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // insert your application logic here
    let user = User {
        id: 1337,
        name: payload.name,
        role: UserRole::Admin,
        email: payload.email,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}
// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
    password:String,
}

#[derive(Serialize)]
enum UserRole{
    Admin,
    User,
}


// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
    email: String,
    role: UserRole,
}
