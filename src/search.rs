use crate::*;
use interface::SearchArgs;

#[async_trait::async_trait]
impl NHRunnable for SearchArgs {
    async fn run(&self) -> Result<()> {
        todo!()
    }
}
