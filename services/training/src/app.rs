use crate::AppState;
use crate::monitoring::health::HealthApi;
use crate::orchestration::{CommandingApi, EventingApi, InboxApi, OutboxApi};
use crate::reporting::ReportingApi;
use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema, http::GraphiQLSource};
use async_graphql_poem::GraphQL;
use poem::{Route, get, handler, web::Html};
use poem_openapi::OpenApiService;

struct Query;

#[Object]
impl Query {
    async fn health(&self) -> &'static str {
        shared::health()
    }
}

#[handler]
async fn graphiql() -> Html<String> {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

pub(crate) fn app(state: AppState, token: String) -> Route {
    let api = OpenApiService::new(
        (
            HealthApi,
            InboxApi::new(state.inbox),
            CommandingApi::new(state.commanding),
            EventingApi::new(state.events),
            OutboxApi::new(state.outbox),
            ReportingApi::new(state.pool, token),
        ),
        "Training",
        env!("CARGO_PKG_VERSION"),
    )
    .server("/api");
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    let mut route = Route::new()
        .nest("/api/docs", api.swagger_ui())
        .at("/api/openapi.json", api.spec_endpoint())
        .nest("/api", api)
        .at("/graphql", GraphQL::new(schema));
    if cfg!(debug_assertions) {
        route = route.at("/graphql/ide", get(graphiql));
    }
    route
}
