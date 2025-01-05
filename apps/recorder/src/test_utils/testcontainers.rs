use async_trait::async_trait;
use bollard::container::ListContainersOptions;
use itertools::Itertools;
use testcontainers::{
    core::logs::consumer::logging_consumer::LoggingConsumer, ContainerRequest, Image, ImageExt,
};

pub const TESTCONTAINERS_PROJECT_KEY: &str = "tech.enfw.testcontainers.project";
pub const TESTCONTAINERS_CONTAINER_KEY: &str = "tech.enfw.testcontainers.container";
pub const TESTCONTAINERS_PRUNE_KEY: &str = "tech.enfw.testcontainers.prune";

#[async_trait]
pub trait ContainerRequestEnhancedExt<I>: Sized + ImageExt<I>
where
    I: Image,
{
    async fn with_prune_existed_label(
        self,
        container_label: &str,
        prune: bool,
        force: bool,
    ) -> color_eyre::eyre::Result<Self>;

    fn with_default_log_consumer(self) -> Self;
}

#[async_trait]
impl<I> ContainerRequestEnhancedExt<I> for ContainerRequest<I>
where
    I: Image,
{
    async fn with_prune_existed_label(
        self,
        container_label: &str,
        prune: bool,
        force: bool,
    ) -> color_eyre::eyre::Result<Self> {
        use std::collections::HashMap;

        use bollard::container::PruneContainersOptions;
        use testcontainers::core::client::docker_client_instance;

        if prune {
            let client = docker_client_instance().await?;

            let mut filters = HashMap::<String, Vec<String>>::new();

            filters.insert(
                String::from("label"),
                vec![
                    format!("{TESTCONTAINERS_PRUNE_KEY}=true"),
                    format!("{}={}", TESTCONTAINERS_PROJECT_KEY, "konobangu"),
                    format!("{}={}", TESTCONTAINERS_CONTAINER_KEY, container_label),
                ],
            );

            if force {
                let result = client
                    .list_containers(Some(ListContainersOptions {
                        all: false,
                        filters: filters.clone(),
                        ..Default::default()
                    }))
                    .await?;

                let remove_containers = result
                    .iter()
                    .filter(|c| matches!(c.state.as_deref(), Some("running")))
                    .flat_map(|c| c.id.as_deref())
                    .collect_vec();

                futures::future::try_join_all(
                    remove_containers
                        .iter()
                        .map(|c| client.stop_container(c, None)),
                )
                .await?;

                tracing::warn!(name = "stop running containers", result = ?remove_containers);
            }

            let result = client
                .prune_containers(Some(PruneContainersOptions { filters }))
                .await?;

            tracing::warn!(name = "prune existed containers", result = ?result);
        }

        let result = self.with_labels([
            (TESTCONTAINERS_PRUNE_KEY, "true"),
            (TESTCONTAINERS_PROJECT_KEY, "konobangu"),
            (TESTCONTAINERS_CONTAINER_KEY, container_label),
        ]);

        Ok(result)
    }

    fn with_default_log_consumer(self) -> Self {
        self.with_log_consumer(
            LoggingConsumer::new()
                .with_stdout_level(log::Level::Info)
                .with_stderr_level(log::Level::Error),
        )
    }
}
