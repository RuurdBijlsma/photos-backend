use color_eyre::Result;
use colored::*;
use crate::helpers::test_context::test_context::TestContext;
use crate::tests::{test_auth, test_health_endpoint};
use std::future::Future;
use std::time::Instant;
use tracing::Level;
use tracing_subscriber::{fmt, EnvFilter};

macro_rules! run_test {
    ($call:expr) => {
        run_test_impl(stringify!($call), $call)
    };
}

/// A helper to make test output distinct and readable
async fn run_test_impl<Fut>(raw_name: &str, test: Fut) -> Result<()>
where
    Fut: Future<Output = Result<()>>,
{
    let name_no_args = raw_name.split('(').next().unwrap_or(raw_name);
    let pretty_name = name_no_args.split("::").last().unwrap_or(name_no_args).trim();

    println!("{}", "─".repeat(60).truecolor(80, 80, 80));
    println!(
        "{} {}",
        " RUNNING ".on_cyan().black().bold(),
        pretty_name.cyan().bold()
    );

    let start_time = Instant::now();
    let result = test.await;
    let elapsed = start_time.elapsed();

    match result {
        Ok(_) => {
            println!(
                "{} {} ({:.2?})",
                " PASSED ".on_green().black().bold(),
                pretty_name.green(),
                elapsed
            );
        }
        Err(ref e) => {
            println!(
                "{} {} ({:.2?})",
                " FAILED ".on_red().black().bold(),
                pretty_name.red(),
                elapsed
            );
            println!("\n{:?}", e);
        }
    }

    result
}

#[tokio::test]
async fn integration_test() -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,hyper=error,reqwest=error".into());

    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_max_level(Level::INFO)
        .compact() // Removes timestamp/levels for cleaner output
        .with_target(false) // Hides the module path (e.g., common_services::api...)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed");

    color_eyre::install().expect("Failed to install color_eyre");

    let context = TestContext::new().await?;
    
    run_test!(test_health_endpoint(&context)).await?;
    run_test!(test_auth(&context)).await?;

    println!();

    println!("{}", "─".repeat(60).truecolor(80, 80, 80));

    Ok(())
}
