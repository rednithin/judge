use async_trait::async_trait;

#[async_trait]
pub trait LanguageExecutor {
    async fn prepare(&self);
    async fn execute(&self);
    async fn teardown(&self);
}

pub struct Rust;
use crate::util;

#[async_trait]
impl LanguageExecutor for Rust {
    async fn prepare(&self) {
        todo!()
    }

    async fn execute(&self) {
        todo!()
    }

    async fn teardown(&self) {
        todo!()
    }
}
