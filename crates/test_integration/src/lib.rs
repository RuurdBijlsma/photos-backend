#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

#[cfg(test)]
pub mod helpers;
#[cfg(test)]
pub mod tests;

#[cfg(test)]
mod runner {
    use std::time::Instant;
use crate::helpers::orchestration_utils::setup_tracing_and_panic_handling;
    use crate::helpers::test_context::test_context::TestContext;
    use crate::tests::*;
    use crate::{execute_suite, run_test};
    use color_eyre::Result;
    use colored::*;

    #[tokio::test]
    async fn integration_suite() -> Result<()> {
        setup_tracing_and_panic_handling();
        let context = TestContext::new().await?;

        execute_suite!(
            &context,
            [
                test_health_endpoint,
                test_auth,
                test_second_register_attempt,
            ]
        );

        Ok(())
    }
}
