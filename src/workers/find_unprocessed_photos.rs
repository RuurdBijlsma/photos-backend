use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

pub struct Worker {
    pub ctx: AppContext,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct WorkerArgs {}

#[async_trait]
impl BackgroundWorker<WorkerArgs> for Worker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }
    async fn perform(&self, _args: WorkerArgs) -> Result<()> {
        println!("=================FindUnprocessedPhotos=======================");
        // TODO: Some actual work goes here...
        Ok(())
    }
}
