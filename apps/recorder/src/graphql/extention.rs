use std::sync::Arc;

use async_graphql::{
    ServerResult, Value,
    extensions::{Extension, ExtensionContext, ExtensionFactory, NextResolve, ResolveInfo},
};

pub struct GraphqlAuthExtension;

#[async_trait::async_trait]
impl Extension for GraphqlAuthExtension {
    async fn resolve(
        &self,
        ctx: &ExtensionContext<'_>,
        info: ResolveInfo<'_>,
        next: NextResolve<'_>,
    ) -> ServerResult<Option<Value>> {
        dbg!(info.field);
        next.run(ctx, info).await
    }
}

impl ExtensionFactory for GraphqlAuthExtension {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(GraphqlAuthExtension)
    }
}
