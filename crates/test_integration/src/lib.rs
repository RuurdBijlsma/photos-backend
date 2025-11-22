#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap
)]

#[cfg(test)]
pub mod runner;
#[cfg(test)]
pub mod test_constants;
#[cfg(test)]
pub mod test_helpers;
#[cfg(test)]
pub mod tests;

#[cfg(test)]
mod test_runner {
    use crate::runner::context::test_context::TestContext;
    use crate::runner::orchestration_utils::setup_tracing_and_panic_handling;
    use crate::tests::test_auth::{
        test_login, test_logout, test_refresh, test_register, test_second_register_attempt,
    };
    use crate::tests::test_root::test_health_endpoint;
    use crate::{execute_suite, run_test};
    use color_eyre::Result;
    use colored::*;
    use std::time::Instant;
    use crate::tests::test_onboarding::{test_onboarding, test_start_processing};

    #[tokio::test]
    async fn integration_suite() -> Result<()> {
        setup_tracing_and_panic_handling();
        let context = TestContext::new().await?;

        execute_suite!(
            &context,
            [
                test_health_endpoint,
                test_register,
                test_second_register_attempt,
                test_login,
                test_refresh,
                test_logout,
                test_onboarding,
                test_start_processing,
            ]
        );

        Ok(())
    }
}
