// use std::{fs, path};
use std::{
    convert::Infallible,
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};

use axum::{
    body::{Bytes, Full},
    handler::get,
    http::{Response, StatusCode},
    response::IntoResponse,
    Json, Router,
};
use futures::ready;
use hyper::server::accept::Accept;
use k8s_openapi::api::core::v1;
use serde::Deserialize;
use serde_json::json;
use thiserror::Error;
use tokio::net::{UnixListener, UnixStream};
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;

use lockc::{
    bpfstructs,
    k8s_agent_api::{Policies, ServerStatus, DEFAULT_SOCKET_PATH},
};

// static APP_NAME: &str = "lockc-runc-wrapper";

static DEFAULT_HEALTH_ADDRESS: &str = "0.0.0.0:8080";

static STATUS_OK: &str = "ok";

static LABEL_POLICY_ENFORCE: &str = "pod-security.kubernetes.io/enforce";
static LABEL_POLICY_ENFORCE_VERSION: &str = "pod-security.kubernetes.io/enforce-version";
static LABEL_POLICY_AUDIT: &str = "pod-security.kubernetes.io/audit";
static LABEL_POLICY_AUDIT_VERSION: &str = "pod-security.kubernetes.io/audit-version";
static LABEL_POLICY_WARN: &str = "pod-security.kubernetes.io/warn";
static LABEL_POLICY_WARN_VERSION: &str = "pod-security.kubernetes-io/warn-version";

static DEFAULT_POLICY_VERSION: &str = "v1.22";

#[derive(Error, Debug)]
enum GetPoliciesError {
    #[error(transparent)]
    Kube(#[from] kube::Error),

    #[error(transparent)]
    PolicyLevel(#[from] bpfstructs::PolicyLevelError),
}

impl IntoResponse for GetPoliciesError {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> Response<Self::Body> {
        let body = Json(json!({"error": self.to_string()}));

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

/// Gets policies and their versions from Kubernetes namespace labels.
async fn get_policies(namespace: &str) -> Result<Policies, GetPoliciesError> {
    // Apply the privileged policy for kube-system containers immediately.
    // Otherwise the core k8s components (apiserver, scheduler) won't be able
    // to run.
    // If container has no k8s namespace, apply the baseline policy.
    if namespace == "kube-system" {
        return Ok(Policies {
            enforce: bpfstructs::container_policy_level_POLICY_LEVEL_PRIVILEGED,
            enforce_version: DEFAULT_POLICY_VERSION.to_string(),
            audit: bpfstructs::container_policy_level_POLICY_LEVEL_PRIVILEGED,
            audit_version: DEFAULT_POLICY_VERSION.to_string(),
            warn: bpfstructs::container_policy_level_POLICY_LEVEL_PRIVILEGED,
            warn_version: DEFAULT_POLICY_VERSION.to_string(),
        });
    }

    let client = kube::Client::try_default().await?;

    let namespaces: kube::api::Api<v1::Namespace> = kube::api::Api::all(client);
    let namespace = namespaces.get(namespace).await?;

    // Try to get policy levels and versions from appropriate labels. If labels
    // do not exist, apply the default values.
    match namespace.metadata.labels {
        Some(v) => Ok(Policies {
            enforce: match v.get(LABEL_POLICY_ENFORCE) {
                Some(p) => bpfstructs::policy_level_from_str(p)?,
                None => bpfstructs::container_policy_level_POLICY_LEVEL_BASELINE,
            },
            enforce_version: match v.get(LABEL_POLICY_ENFORCE_VERSION) {
                Some(ver) => ver.to_string(),
                None => DEFAULT_POLICY_VERSION.to_string(),
            },
            audit: match v.get(LABEL_POLICY_AUDIT) {
                Some(p) => bpfstructs::policy_level_from_str(p)?,
                None => bpfstructs::container_policy_level_POLICY_LEVEL_BASELINE,
            },
            audit_version: match v.get(LABEL_POLICY_AUDIT_VERSION) {
                Some(ver) => ver.to_string(),
                None => DEFAULT_POLICY_VERSION.to_string(),
            },
            warn: match v.get(LABEL_POLICY_WARN) {
                Some(p) => bpfstructs::policy_level_from_str(p)?,
                None => bpfstructs::container_policy_level_POLICY_LEVEL_BASELINE,
            },
            warn_version: match v.get(LABEL_POLICY_WARN_VERSION) {
                Some(ver) => ver.to_string(),
                None => DEFAULT_POLICY_VERSION.to_string(),
            },
        }),
        None => Ok(Policies {
            enforce: bpfstructs::container_policy_level_POLICY_LEVEL_BASELINE,
            enforce_version: DEFAULT_POLICY_VERSION.to_string(),
            audit: bpfstructs::container_policy_level_POLICY_LEVEL_BASELINE,
            audit_version: DEFAULT_POLICY_VERSION.to_string(),
            warn: bpfstructs::container_policy_level_POLICY_LEVEL_BASELINE,
            warn_version: DEFAULT_POLICY_VERSION.to_string(),
        }),
    }
}

async fn handler_index() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ServerStatus {
            status: STATUS_OK.to_string(),
        }),
    )
}

#[derive(Deserialize)]
struct GetPolicies {
    namespace: String,
}

async fn handler_policies(
    Json(payload): Json<GetPolicies>,
) -> Result<Json<Policies>, GetPoliciesError> {
    let policies = get_policies(&payload.namespace).await?;
    Ok(Json(policies))
}

// #[get("/healthz")]
// async fn health() -> Result<HttpResponse, Error> {
//     Ok(HttpResponse::Ok().json(ServerStatus {
//         status: STATUS_OK.to_string(),
//     }))
// }

struct ServerAccept {
    uds: UnixListener,
}

impl Accept for ServerAccept {
    type Conn = UnixStream;
    type Error = BoxError;

    fn poll_accept(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        let (stream, _addr) = ready!(self.uds.poll_accept(cx))?;
        Poll::Ready(Some(Ok(stream)))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let middleware_stack = ServiceBuilder::new()
        // `TraceLayer` adds high level tracing and logging
        .layer(TraceLayer::new_for_http())
        .into_inner();

    // build our application with a route
    let app = Router::new()
        .route("/", get(handler_index))
        .route("/policies", get(handler_policies))
        .layer(middleware_stack.clone());

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    // let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", DEFAULT_SOCKET_PATH);
    let uds = UnixListener::bind(DEFAULT_SOCKET_PATH)?;
    let uds_server = axum::Server::builder(ServerAccept { uds }).serve(app.into_make_service());
    let uds_server_task = tokio::spawn(uds_server);
    // .await?;

    let health_app = Router::new()
        .route("/healthz", get(handler_index))
        .layer(middleware_stack);
    let addr: SocketAddr = DEFAULT_HEALTH_ADDRESS.parse()?;
    tracing::info!("listening on {}", DEFAULT_HEALTH_ADDRESS);
    let healthz_server = axum::Server::bind(&addr).serve(health_app.into_make_service());
    let healthz_server_task = tokio::spawn(healthz_server);

    uds_server_task.await??;
    healthz_server_task.await??;
    // .await?;

    Ok(())
}
