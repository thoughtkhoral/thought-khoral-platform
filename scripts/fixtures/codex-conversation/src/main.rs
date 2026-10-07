// SPDX-License-Identifier: Apache-2.0
//! Task9 synthetic composed-stack fixture. The three production libraries are
//! linked as exact local path dependencies by the outer runner.
use axum::{
    Json, Router,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::post,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use chrono::{Duration as ChronoDuration, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode, jwk::{Jwk, JwkSet, PublicKeyUse}};
use rand::thread_rng;
use reqwest::Client;
use rsa::{RsaPrivateKey, pkcs1::EncodeRsaPrivateKey};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::{PgPool, Row};
use std::{
    collections::BTreeMap,
    env,
    ffi::OsString,
    future::Future,
    net::{IpAddr, Ipv4Addr},
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
    time::Duration,
};
use thought_khoral_agent_gateway::{
    GatewayConfig, RoomGatewayClient,
    conversation_catalog::catalog_service,
    conversation_dispatcher::{ConversationDispatcher, WorkerClient},
    conversation_validation::Admission,
};
use thought_khoral_codex_agent::{
    a2a_service::{ServiceConfig, service},
    config::Config,
    worker::Worker,
};
use thought_khoral_room_gateway::{
    AuthValidator, GatewayState, NewEvent, WebSocketPolicy, app, append_event, events_after,
    conversation_protocol::{CODEX_AGENT_ID, ConversationError, PROFILE_VERSION},
    conversation_service::CatalogQuery,
    conversation_store::ConversationPolicy,
};
use tokio::{net::TcpListener, process::{Child, Command}, time::sleep};
use uuid::Uuid;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
const ISSUER: &str = "http://keycloak.test/realms/thought-khoral";
const AUDIENCE: &str = "thought-khoral-room-gateway";
const INVOCATION_SECRET: &str = "Task9-synthetic-worker-invocation";
const BRIDGE_SECRET: &str = "Task9-synthetic-catalog-bridge-secret-001";
const CLIENT_SECRET: &str = "Task9-synthetic-oidc-client-secret";
const REF_SECRET: &str = "Task9-synthetic-reference-secret";
const KID: &str = "Task9-synthetic-issuer";

#[derive(Clone)]
struct Issuer(EncodingKey);
#[derive(Serialize)]
struct Claims {
    sub: String,
    iss: &'static str,
    aud: &'static str,
    exp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    n2n_role: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    azp: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    iat: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'static str>,
}
impl Issuer {
    fn token(&self, sub: String, role: Option<&'static str>, name: Option<&'static str>) -> String {
        let now = Utc::now().timestamp();
        let claims = Claims {
            sub,
            iss: ISSUER,
            aud: AUDIENCE,
            exp: now + if role.is_none() { 300 } else { 3600 },
            n2n_role: role,
            azp: if role.is_none() { Some("thought-khoral-agent-gateway") } else { None },
            iat: if role.is_none() { Some(now) } else { None },
            name,
        };
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(KID.into());
        encode(&header, &claims, &self.0).expect("synthetic signing key")
    }
}
async fn token_endpoint(State(issuer): State<Issuer>, headers: HeaderMap, body: String) -> impl IntoResponse {
    let valid = headers.get("authorization").and_then(|v| v.to_str().ok())
        == Some(format!("Basic {}", STANDARD.encode(format!("thought-khoral-agent-gateway:{CLIENT_SECRET}"))).as_str());
    if !valid || body != "grant_type=client_credentials" {
        return (StatusCode::UNAUTHORIZED, Json(json!({"error":"invalid_client"}))).into_response();
    }
    (StatusCode::OK, Json(json!({"access_token":issuer.token("service-account-thought-khoral-agent-gateway".into(), None, None),"token_type":"Bearer","expires_in":300}))).into_response()
}

struct LoopbackCatalog(Client);
impl CatalogQuery for LoopbackCatalog {
    fn page<'a>(&'a self, _agent: Uuid, _cursor: Option<String>, _limit: usize)
        -> Pin<Box<dyn Future<Output=std::result::Result<Value,ConversationError>> + Send + 'a>> {
        Box::pin(async move {
            let response = self.0.get("http://127.0.0.1:9092/internal/agent-conversations/v1/models")
                .bearer_auth(BRIDGE_SECRET).send().await.map_err(|_| ConversationError::RuntimeUnavailable)?;
            if !response.status().is_success() { return Err(ConversationError::RuntimeUnavailable); }
            response.json().await.map_err(|_| ConversationError::RuntimeUnavailable)
        })
    }
}
#[derive(Clone, Copy, Debug)]
enum GatePhase { BeforeCommit, AfterCommit }
#[derive(Clone)]
struct UpdateGate {
    target: Arc<tokio::sync::Mutex<Option<(String, GatePhase)>>>,
    directory: PathBuf,
}
impl UpdateGate {
    fn new(directory: PathBuf) -> Self {
        Self { target: Arc::new(tokio::sync::Mutex::new(None)), directory }
    }
    async fn arm(&self, task: &str, phase: GatePhase) {
        *self.target.lock().await = Some((task.into(), phase));
    }
    fn marker(&self, task: &str) -> PathBuf { self.directory.join(format!("broker-update-gate-{task}")) }
}
async fn gate_update(State(gate): State<UpdateGate>, request: Request, next: Next) -> Response {
    let path = request.uri().path().to_owned();
    let armed = {
        let mut target = gate.target.lock().await;
        if target.as_ref().is_some_and(|(task, _)| path == format!("/internal/agent-conversations/v1/tasks/{task}/updates")) {
            target.take()
        } else { None }
    };
    if let Some((task, GatePhase::BeforeCommit)) = &armed {
        std::fs::write(gate.marker(task), b"before-commit").expect("Task9 gate marker");
        std::future::pending::<()>().await;
    }
    let response = next.run(request).await;
    if let Some((task, GatePhase::AfterCommit)) = &armed {
        std::fs::write(gate.marker(task), b"after-commit").expect("Task9 gate marker");
        std::future::pending::<()>().await;
    }
    response
}
async fn hold_send(request: Request, next: Next) -> Response {
    if request.uri().path() == "/" && request.method() == axum::http::Method::POST {
        std::fs::write(state().join("native-requests.jsonl.hold_send.stage"), b"submission-intent").expect("Task9 hold-send marker");
        std::future::pending::<()>().await;
    }
    next.run(request).await
}
fn admission() -> Admission {
    Admission::new("catalog-1".into(), "fixed-1".into(),
        BTreeMap::from([
            ("model-a".into(), vec!["effort-medium".into(), "effort-high".into()]),
            ("model-b".into(), vec!["effort-medium".into(), "effort-high".into()]),
        ])).unwrap()
}
fn gateway_config() -> GatewayConfig {
    GatewayConfig::parse(BTreeMap::from([
        ("THOUGHT_KHORAL_ROOM_GATEWAY_ORIGIN".into(), "http://thought-khoral-room-gateway:8080/".into()),
        ("THOUGHT_KHORAL_KEYCLOAK_TOKEN_URL".into(), "http://thought-khoral-keycloak:8080/realms/thought-khoral/protocol/openid-connect/token".into()),
        ("THOUGHT_KHORAL_AGENT_GATEWAY_CLIENT_ID".into(), "thought-khoral-agent-gateway".into()),
        ("THOUGHT_KHORAL_AGENT_GATEWAY_CLIENT_SECRET".into(), CLIENT_SECRET.into()),
        ("THOUGHT_KHORAL_REFERENCE_AGENT_INBOUND_SECRET".into(), REF_SECRET.into()),
        ("THOUGHT_KHORAL_REFERENCE_AGENT_CARD_URL".into(), "http://127.0.0.1:9090/.well-known/agent-card.json".into()),
        ("THOUGHT_KHORAL_ALLOWED_HANDOFF_HOSTS".into(), "reference.test".into()),
        ("THOUGHT_KHORAL_REFERENCE_AGENT_HANDOFF_HOST".into(), "reference.test".into()),
        ("THOUGHT_KHORAL_AGENT_POLL_MILLIS".into(), "100".into()),
    ])).unwrap()
}
fn loopback_worker() -> WorkerClient {
    WorkerClient::with_loopback_dns(INVOCATION_SECRET.into(), admission(),
        IpAddr::V4(Ipv4Addr::LOCALHOST)).unwrap()
}
fn state() -> PathBuf { PathBuf::from(env::var("TASK9_STATE_DIR").expect("Task9 state")) }

async fn worker_child() -> Result<()> {
    let state = state();
    let cwd = state.join("workspace");
    let home = state.join("native");
    std::fs::create_dir_all(&cwd)?;
    std::fs::create_dir_all(&home)?;
    let mut config = Config::new(PathBuf::from("/usr/bin/python3"), cwd, home);
    config.tool_catalog = PathBuf::from(env::var("TASK9_WORKER_REPO")?)
        .join("contracts/codex-app-server-0.160.0/restricted-models.json");
    config.arguments = vec![
        OsString::from(env::var("TASK9_FAKE_APP_SERVER")?),
        "--scenario".into(),
        env::var("TASK9_SCENARIO").unwrap_or_else(|_| "usage".into()).into(),
        "--capture".into(),
        state.join("native-requests.jsonl").into_os_string(),
        "--persistent".into(),
    ];
    config.model_allowlist = vec!["model-a".into(), "model-b".into()];
    config.deadline = Duration::from_secs(15);
    config.interrupt_grace = Duration::from_millis(100);
    let worker = Worker::open(config, &state.join("receipts.sqlite")).await?;
    let mut router = service(worker, ServiceConfig::new(INVOCATION_SECRET.into(), Utc::now() + ChronoDuration::hours(1)))?;
    if env::var("TASK9_SCENARIO").as_deref() == Ok("hold_send") {
        router = router.layer(middleware::from_fn(hold_send));
    }
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 9091)).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
async fn mediator_once() -> Result<()> {
    let config = gateway_config();
    let room = RoomGatewayClient::from_config_with_loopback_dns(&config, IpAddr::V4(Ipv4Addr::LOCALHOST))?;
    let dispatcher = ConversationDispatcher::new(room, loopback_worker(), state().join("mediator"), "Task9-mediator".into())?;
    let processed = dispatcher.run_once().await?;
    println!("Task9 mediator processed={processed}");
    Ok(())
}
fn policy() -> ConversationPolicy {
    serde_json::from_value(json!({"enabled":true,"policyRevision":"Task9-policy-1","guidanceRevision":"fixed-1","catalogRevision":"catalog-1","model":"model-a","reasoningEffort":"effort-medium","models":[{"id":"model-a","reasoningEfforts":["effort-medium","effort-high"]},{"id":"model-b","reasoningEfforts":["effort-medium","effort-high"]}],"turnDeadlineSeconds":5,"interruptGraceSeconds":1})).unwrap()
}
fn new_event(room: Uuid, actor: Uuid, name: &str, text: &str, targeted: bool) -> NewEvent {
    NewEvent { room_id: room, request_id: Uuid::new_v4(), event_type: "message.created".into(),
        actor_id: actor, actor_role: "human".into(), actor_display_name: Some(name.into()),
        payload: json!({"text":text,"delivery":if targeted {"mentioned"} else {"room"},"mentions":[],"audienceIds":if targeted {vec![actor]} else {vec![]}}),
        occurred_at: Utc::now() }
}
fn turn(room: Uuid, text: &str, conversation: Value, settings: Option<Value>) -> Value {
    let mut value = json!({"profileVersion":PROFILE_VERSION,"requestId":Uuid::new_v4(),"roomId":room,"agentId":CODEX_AGENT_ID,
        "occurredAt":Utc::now(),"text":text,"mentions":[{"type":"participant","id":CODEX_AGENT_ID,"token":"codex-agent"}],"conversation":conversation});
    if let Some(settings) = settings { value["settings"] = settings; }
    value
}
async fn post_json(client: &Client, path: &str, token: &str, body: &Value) -> Result<(reqwest::StatusCode,Value)> {
    let response = client.post(format!("http://127.0.0.1:8080{path}"))
        .bearer_auth(token).json(body).send().await?;
    let status = response.status();
    Ok((status, response.json().await.unwrap_or(Value::Null)))
}
async fn get(client: &Client, path: &str, token: &str) -> Result<(reqwest::StatusCode,Value)> {
    let response = client.get(format!("http://127.0.0.1:8080{path}"))
        .bearer_auth(token).send().await?;
    let status = response.status();
    Ok((status, response.json().await.unwrap_or(Value::Null)))
}
fn native_records(path: &Path) -> Result<Vec<Value>> {
    Ok(std::fs::read_to_string(path)?.lines().map(serde_json::from_str).collect::<std::result::Result<_,_>>()?)
}
async fn run_mediator(expect_success: bool) -> Result<()> {
    let status = Command::new(env::current_exe()?).arg("--mediator-once").status().await?;
    if status.success() != expect_success { return Err(format!("Task9 mediator child exited {status}; expected success={expect_success}").into()); }
    Ok(())
}
async fn wait_for_worker(client: &Client) -> Result<()> {
    for _ in 0..100 {
        if client.get("http://127.0.0.1:9091/.well-known/agent-card.json")
            .bearer_auth(INVOCATION_SECRET).send().await.is_ok_and(|r| r.status().is_success()) { return Ok(()); }
        sleep(Duration::from_millis(50)).await;
    }
    Err("Task9 worker child did not start".into())
}
async fn wait_for_file(path: &Path) -> Result<()> {
    for _ in 0..200 {
        if path.is_file() { return Ok(()); }
        sleep(Duration::from_millis(50)).await;
    }
    Err(format!("Task9 checkpoint missing: {}", path.display()).into())
}
async fn stop_child(child: &mut Child) -> Result<()> {
    child.kill().await?;
    child.wait().await?;
    Ok(())
}
async fn spawn_worker(client: &Client, scenario: &str) -> Result<Child> {
    let child = Command::new(env::current_exe()?).arg("--worker-child")
        .env("TASK9_SCENARIO", scenario).kill_on_drop(true).spawn()?;
    wait_for_worker(client).await?;
    Ok(child)
}
async fn worker_boundary(
    client: &Client, pool: &PgPool, issuer: &Issuer, worker: &mut Child,
    scenario: &str, expected_phase: Option<&str>, expect_thread: bool, expect_turn: bool,
) -> Result<()> {
    stop_child(worker).await?;
    *worker = spawn_worker(client, scenario).await?;
    let room = Uuid::new_v4();
    let human = Uuid::new_v4();
    let token = issuer.token(human.to_string(), Some("human"), Some("Maya"));
    let input = turn(room, "@codex-agent Task9 boundary probe.", json!({"mode":"new"}), None);
    let (status, accepted) = post_json(client, "/api/agent-conversations/v1/turns", &token, &input).await?;
    assert_eq!(status, reqwest::StatusCode::ACCEPTED, "{scenario}: {accepted}");
    let before = native_records(&state().join("native-requests.jsonl"))?.iter()
        .filter(|r| r["request"]["method"] == "turn/start").count();
    let mut mediator = Command::new(env::current_exe()?).arg("--mediator-once").kill_on_drop(true).spawn()?;
    let marker = state().join(format!("native-requests.jsonl.{scenario}.stage"));
    wait_for_file(&marker).await?;
    let mediator_record: Value = serde_json::from_slice(&std::fs::read(state().join("mediator").join(format!("{}.json", accepted["taskId"].as_str().unwrap())))?)?;
    assert_eq!(mediator_record["phase"], "submission-intent", "{scenario}");
    let sqlite = sqlx::SqlitePool::connect(&format!("sqlite://{}?mode=rw", state().join("receipts.sqlite").display())).await?;
    let receipt = sqlx::query("SELECT phase,thread_id,turn_id FROM receipts WHERE task_id=?")
        .bind(accepted["taskId"].as_str().unwrap()).fetch_optional(&sqlite).await?;
    if let Some(expected_phase) = expected_phase {
        let receipt = receipt.expect("Task9 worker receipt must exist");
        assert_eq!(receipt.get::<String,_>("phase"), expected_phase, "{scenario}");
        assert_eq!(receipt.get::<Option<String>,_>("thread_id").is_some(), expect_thread, "{scenario}");
        assert_eq!(receipt.get::<Option<String>,_>("turn_id").is_some(), expect_turn, "{scenario}");
    } else { assert!(receipt.is_none(), "{scenario} reached worker receipt before submission"); }
    sqlite.close().await;
    stop_child(&mut mediator).await?;
    stop_child(worker).await?;
    if scenario != "hold_send" {
        let native_pid = native_records(&state().join("native-requests.jsonl"))?.last().unwrap()["pid"].as_i64().unwrap() as i32;
        let killed = unsafe { libc::kill(-native_pid, libc::SIGKILL) };
        assert_eq!(killed, 0, "Task9 must reap its held native fixture group");
    }
    *worker = spawn_worker(client, "usage").await?;
    let response = client.get(format!("http://127.0.0.1:9091/control/v1/receipts/{}", accepted["taskId"].as_str().unwrap()))
        .bearer_auth(INVOCATION_SECRET).send().await?;
    if expected_phase.is_some() {
        let receipt: Value = response.json().await?;
        assert_eq!(receipt["phase"], "interrupted", "{scenario}: {receipt}");
    } else {
        assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST, "{scenario} unexpectedly created a receipt");
    }
    let expires_at: chrono::DateTime<Utc> = sqlx::query_scalar("SELECT expires_at FROM conversation_tasks WHERE task_id=$1")
        .bind(Uuid::parse_str(accepted["taskId"].as_str().unwrap())?).fetch_one(pool).await?;
    let until = (expires_at - Utc::now()).to_std().unwrap_or_default();
    sleep(until + Duration::from_millis(100)).await;
    run_mediator(true).await?;
    let (_, task) = get(client, &format!("/api/agent-conversations/v1/rooms/{room}/tasks/{}", accepted["taskId"].as_str().unwrap()), &token).await?;
    assert_eq!(task["state"], "failed", "{scenario}: {task}");
    assert!(task["replyEventId"].is_null());
    assert_eq!(events_after(pool, room, 0).await?.into_iter().filter(|e| e.actor_id == CODEX_AGENT_ID).count(), 0);
    let after = native_records(&state().join("native-requests.jsonl"))?.iter()
        .filter(|r| r["request"]["method"] == "turn/start").count();
    assert_eq!(after - before, if matches!(scenario, "hold_send" | "hold_thread_start") { 0 } else { 1 }, "{scenario} retried native submission");
    println!("Task9 worker boundary {scenario}: initial receipt {expected_phase:?}, native turn delta {}, public replies 0", after - before);
    Ok(())
}
async fn mediator_boundary(
    client: &Client, pool: &PgPool, issuer: &Issuer, gate: &UpdateGate, phase: GatePhase,
) -> Result<()> {
    let room = Uuid::new_v4();
    let human = Uuid::new_v4();
    let token = issuer.token(human.to_string(), Some("human"), Some("Leo"));
    let (status, accepted) = post_json(client, "/api/agent-conversations/v1/turns", &token,
        &turn(room, "@codex-agent Task9 commit boundary.", json!({"mode":"new"}), None)).await?;
    assert_eq!(status, reqwest::StatusCode::ACCEPTED, "{accepted}");
    let task = accepted["taskId"].as_str().unwrap();
    gate.arm(task, phase).await;
    let before = native_records(&state().join("native-requests.jsonl"))?.iter()
        .filter(|r| r["request"]["method"] == "turn/start").count();
    let mut mediator = Command::new(env::current_exe()?).arg("--mediator-once").kill_on_drop(true).spawn()?;
    wait_for_file(&gate.marker(task)).await?;
    let worker_receipt: Value = client.get(format!("http://127.0.0.1:9091/control/v1/receipts/{task}"))
        .bearer_auth(INVOCATION_SECRET).send().await?.json().await?;
    assert_eq!(worker_receipt["phase"], "completed", "{phase:?}");
    assert!(worker_receipt["acknowledgement"].is_null(), "{phase:?}");
    let mediator_record: Value = serde_json::from_slice(&std::fs::read(state().join("mediator").join(format!("{task}.json")))?)?;
    assert_eq!(mediator_record["phase"], "completed", "{phase:?}");
    let broker_state: String = sqlx::query_scalar("SELECT state FROM conversation_tasks WHERE task_id=$1")
        .bind(Uuid::parse_str(task)?).fetch_one(pool).await?;
    assert_eq!(broker_state, match phase { GatePhase::BeforeCommit => "running", GatePhase::AfterCommit => "completed" });
    stop_child(&mut mediator).await?;
    run_mediator(true).await?;
    let (_, view) = get(client, &format!("/api/agent-conversations/v1/rooms/{room}/tasks/{task}"), &token).await?;
    assert_eq!(view["state"], "completed", "{phase:?}: {view}");
    assert!(!view["replyEventId"].is_null());
    let worker_receipt: Value = client.get(format!("http://127.0.0.1:9091/control/v1/receipts/{task}"))
        .bearer_auth(INVOCATION_SECRET).send().await?.json().await?;
    assert_eq!(worker_receipt["acknowledgement"]["replyEventId"], view["replyEventId"]);
    let mediator_record: Value = serde_json::from_slice(&std::fs::read(state().join("mediator").join(format!("{task}.json")))?)?;
    assert_eq!(mediator_record["phase"], "acknowledged");
    assert_eq!(events_after(pool, room, 0).await?.into_iter().filter(|e| e.actor_id == CODEX_AGENT_ID).count(), 1);
    let after = native_records(&state().join("native-requests.jsonl"))?.iter()
        .filter(|r| r["request"]["method"] == "turn/start").count();
    assert_eq!(after - before, 1, "{phase:?} repeated native turn");
    println!("Task9 mediator boundary {phase:?}: broker {broker_state} -> completed, native turn delta 1, public replies 1");
    Ok(())
}
async fn migrate(pool: &PgPool, broker: &Path) -> Result<()> {
    let mut paths = std::fs::read_dir(broker.join("migrations"))?
        .map(|e| e.map(|v| v.path())).collect::<std::result::Result<Vec<_>,_>>()?;
    paths.retain(|p| p.extension().is_some_and(|e| e == "sql"));
    paths.sort();
    for path in paths { sqlx::raw_sql(&std::fs::read_to_string(path)?).execute(pool).await?; }
    Ok(())
}
async fn parent() -> Result<()> {
    // Fail before outbound fixture requests if a fixed production authority is occupied.
    let broker_listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 8080)).await?;
    let catalog_listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 9092)).await?;
    let worker_preflight = TcpListener::bind((Ipv4Addr::LOCALHOST, 9091)).await?;
    drop(worker_preflight);
    let pool = PgPool::connect(&env::var("TASK9_DATABASE_URL")?).await?;
    migrate(&pool, Path::new(&env::var("TASK9_BROKER_REPO")?)).await?;
    let private = RsaPrivateKey::new(&mut thread_rng(), 2048)?;
    let der = private.to_pkcs1_der()?;
    let issuer = Issuer(EncodingKey::from_rsa_der(der.as_bytes()));
    let mut jwk = Jwk::from_encoding_key(&issuer.0, Algorithm::RS256)?;
    jwk.common.key_id = Some(KID.into());
    jwk.common.public_key_use = Some(PublicKeyUse::Signature);
    let jwks = serde_json::to_string(&JwkSet { keys: vec![jwk] })?;
    let auth = AuthValidator::new(ISSUER, AUDIENCE, &jwks)?;
    let gateway_state = GatewayState::with_websocket_policy(pool.clone(), auth,
        WebSocketPolicy::new(["http://workspace.test"], Duration::from_secs(5))?);
    let client = Client::builder().no_proxy().redirect(reqwest::redirect::Policy::none()).build()?;
    gateway_state.configure_conversations(policy(), Some(Arc::new(LoopbackCatalog(client.clone())))).await?;
    let token_route = Router::new().route("/realms/thought-khoral/protocol/openid-connect/token", post(token_endpoint)).with_state(issuer.clone());
    let gate = UpdateGate::new(state());
    let broker_router = app(gateway_state).merge(token_route).layer(middleware::from_fn_with_state(gate.clone(), gate_update));
    let broker_task = tokio::spawn(async move { axum::serve(broker_listener, broker_router).await });
    let catalog_router = catalog_service(loopback_worker(), BRIDGE_SECRET.into())?;
    let catalog_task = tokio::spawn(async move { axum::serve(catalog_listener, catalog_router).await });
    let mut worker = spawn_worker(&client, "usage").await?;
    for (label, url, secret) in [
        ("worker catalog", "http://127.0.0.1:9091/control/v1/models", INVOCATION_SECRET),
        ("mediator catalog", "http://127.0.0.1:9092/internal/agent-conversations/v1/models", BRIDGE_SECRET),
    ] {
        let response = client.get(url).bearer_auth(secret).send().await?;
        assert_eq!(response.status(), reqwest::StatusCode::OK, "{label}");
        let catalog: Value = response.json().await?;
        assert_eq!(catalog["data"].as_array().unwrap().len(), 2, "{label}");
    }
    let room = Uuid::new_v4();
    let other_room = Uuid::new_v4();
    let maya = Uuid::new_v4();
    let leo = Uuid::new_v4();
    let public = append_event(&pool, new_event(room, maya, "Maya", "Task9 Maya proposed green.", false)).await?.event_id;
    append_event(&pool, new_event(room, leo, "Leo", "Task9 targeted secret: violet.", true)).await?;
    let correction = append_event(&pool, new_event(room, leo, "Leo", "Task9 Leo corrected the timing to Friday.", false)).await?.event_id;
    append_event(&pool, new_event(other_room, maya, "Maya", "Task9 other-room fact: orange.", false)).await?;
    let token = issuer.token(leo.to_string(), Some("human"), Some("Leo"));
    let first = turn(room, "@codex-agent What did Maya propose?", json!({"mode":"new"}), None);
    let (first_result, replay_result) = tokio::join!(
        post_json(&client, "/api/agent-conversations/v1/turns", &token, &first),
        post_json(&client, "/api/agent-conversations/v1/turns", &token, &first),
    );
    let (status, accepted) = first_result?;
    assert_eq!(status, reqwest::StatusCode::ACCEPTED, "{accepted}");
    let (replay_status, replay) = replay_result?;
    assert_eq!(replay_status, status);
    assert_eq!(replay, accepted, "duplicate request must return the same task");
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM conversation_tasks WHERE room_id=$1").bind(room).fetch_one(&pool).await?;
    assert_eq!(count, 1, "duplicate created a second logical task");
    let frozen: Value = sqlx::query("SELECT frozen_input FROM conversation_tasks WHERE task_id=$1")
        .bind(Uuid::parse_str(accepted["taskId"].as_str().unwrap())?).fetch_one(&pool).await?.get("frozen_input");
    let entries = frozen["context"]["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0]["eventId"], json!(public));
    assert_eq!(entries[1]["eventId"], json!(correction));
    assert_eq!(entries[2]["eventId"], accepted["triggerEventId"]);
    assert_eq!(frozen["triggerEventId"], accepted["triggerEventId"]);
    assert!(!frozen.to_string().contains("violet"));
    assert!(!frozen.to_string().contains("orange"));
    assert_eq!(frozen["context"]["baseRevision"], 0);
    run_mediator(true).await?;
    let task_path = format!("/api/agent-conversations/v1/rooms/{room}/tasks/{}", accepted["taskId"].as_str().unwrap());
    let (status, view) = get(&client, &task_path, &token).await?;
    assert_eq!(status, reqwest::StatusCode::OK);
    assert_eq!(view["state"], "completed", "{view}");
    assert!(!view.to_string().contains("violet") && !view.to_string().contains("orange"));
    let events = events_after(&pool, room, 0).await?;
    assert_eq!(events.iter().filter(|e| e.actor_id == CODEX_AGENT_ID).count(), 1);
    assert_eq!(events.last().unwrap().event_id.to_string(), view["replyEventId"]);
    let records = native_records(&state().join("native-requests.jsonl"))?;
    let first_turns = records.iter().filter(|r| r["request"]["method"] == "turn/start").collect::<Vec<_>>();
    assert_eq!(first_turns.len(), 1);
    let prompt = first_turns[0]["request"]["params"]["input"][0]["text"].as_str().unwrap();
    assert!(prompt.contains("Task9 Maya proposed green."));
    assert!(prompt.contains("Task9 Leo corrected the timing to Friday."));
    assert_eq!(prompt.matches("@codex-agent What did Maya propose?").count(), 1);
    assert!(!prompt.contains("violet") && !prompt.contains("orange"));
    let receipt = std::fs::read_to_string(state().join("mediator").join(format!("{}.json", accepted["taskId"].as_str().unwrap())))?;
    assert!(receipt.contains("acknowledged"));

    let intervening = append_event(&pool, new_event(room, maya, "Maya", "Task9 intervening discussion: budget is twelve.", false)).await?.event_id;
    let second = turn(room, "@codex-agent What changed?", json!({"mode":"continue","id":accepted["conversationId"],"generation":accepted["generation"]}),
        Some(json!({"model":"model-b","reasoningEffort":"effort-high","catalogRevision":"catalog-1"})));
    let (status, continued) = post_json(&client, "/api/agent-conversations/v1/turns", &token, &second).await?;
    assert_eq!(status, reqwest::StatusCode::ACCEPTED, "{continued}");
    let delta: Value = sqlx::query("SELECT frozen_input FROM conversation_tasks WHERE task_id=$1")
        .bind(Uuid::parse_str(continued["taskId"].as_str().unwrap())?).fetch_one(&pool).await?.get("frozen_input");
    assert_eq!(delta["conversation"]["id"], accepted["conversationId"]);
    assert_eq!(delta["context"]["baseRevision"], frozen["context"]["revision"]);
    assert_eq!(delta["context"]["entries"][0]["eventId"], json!(intervening));
    assert_eq!(delta["context"]["entries"][1]["eventId"], continued["triggerEventId"]);
    assert!(!delta.to_string().contains("violet"));
    let wrong = turn(other_room, "@codex-agent Cross-room bind?", json!({"mode":"continue","id":accepted["conversationId"],"generation":accepted["generation"]}), None);
    let (wrong_status, _) = post_json(&client, "/api/agent-conversations/v1/turns", &token, &wrong).await?;
    assert!(!wrong_status.is_success(), "another room bound the conversation");
    worker.kill().await?;
    worker.wait().await?;
    worker = spawn_worker(&client, "usage").await?;
    run_mediator(true).await?;
    let (status, second_view) = get(&client, &format!("/api/agent-conversations/v1/rooms/{room}/tasks/{}", continued["taskId"].as_str().unwrap()), &token).await?;
    assert_eq!(status, reqwest::StatusCode::OK);
    assert_eq!(second_view["state"], "completed", "{second_view}");
    let records = native_records(&state().join("native-requests.jsonl"))?;
    assert_eq!(records.iter().filter(|r| r["request"]["method"] == "turn/start").count(), 2);
    assert_eq!(records.iter().filter(|r| r["request"]["method"] == "thread/start").count(), 1);
    assert_eq!(records.iter().filter(|r| r["request"]["method"] == "thread/resume").count(), 1);
    assert_eq!(second_view["selectedSettings"]["reasoningEffort"], "effort-high");
    assert_eq!(second_view["effectiveSettings"]["reasoningEffort"], "effort-high");
    assert_eq!(second_view["selectedSettings"]["model"], "model-b");
    assert_eq!(second_view["effectiveSettings"]["model"], "model-b");
    assert_eq!(second_view["usage"]["lastTotalTokens"], 40);
    assert_eq!(second_view["usage"]["modelContextWindow"], 100);
    let replies = events_after(&pool, room, 0).await?.into_iter().filter(|e| e.actor_id == CODEX_AGENT_ID).count();
    assert_eq!(replies, 2);

    let reset = turn(room, "@codex-agent Start a fresh thread.", json!({"mode":"new"}), None);
    let (status, reset_accepted) = post_json(&client, "/api/agent-conversations/v1/turns", &token, &reset).await?;
    assert_eq!(status, reqwest::StatusCode::ACCEPTED, "{reset_accepted}");
    assert_ne!(reset_accepted["conversationId"], accepted["conversationId"]);
    assert_eq!(reset_accepted["selectedSettings"]["model"], "model-a");
    assert_eq!(reset_accepted["selectedSettings"]["reasoningEffort"], "effort-medium");
    let reset_frozen: Value = sqlx::query("SELECT frozen_input FROM conversation_tasks WHERE task_id=$1")
        .bind(Uuid::parse_str(reset_accepted["taskId"].as_str().unwrap())?).fetch_one(&pool).await?.get("frozen_input");
    assert_eq!(reset_frozen["context"]["baseRevision"], 0);
    assert!(reset_frozen.to_string().contains("Task9 Maya proposed green."));
    assert!(reset_frozen["context"]["nativeReplyBindings"].as_array().unwrap().is_empty());
    assert!(!reset_frozen.to_string().contains("violet") && !reset_frozen.to_string().contains("orange"));
    let conversation_path = format!("/api/agent-conversations/v1/rooms/{room}/agents/{CODEX_AGENT_ID}");
    let (status, before_reset_view) = get(&client, &conversation_path, &token).await?;
    assert_eq!(status, reqwest::StatusCode::OK);
    assert!(before_reset_view["conversation"]["usage"].is_null(), "fresh session must reset usage to unavailable");
    run_mediator(true).await?;
    let first_receipt: Value = client.get(format!("http://127.0.0.1:9091/control/v1/receipts/{}", accepted["taskId"].as_str().unwrap()))
        .bearer_auth(INVOCATION_SECRET).send().await?.json().await?;
    let continued_receipt: Value = client.get(format!("http://127.0.0.1:9091/control/v1/receipts/{}", continued["taskId"].as_str().unwrap()))
        .bearer_auth(INVOCATION_SECRET).send().await?.json().await?;
    let reset_receipt: Value = client.get(format!("http://127.0.0.1:9091/control/v1/receipts/{}", reset_accepted["taskId"].as_str().unwrap()))
        .bearer_auth(INVOCATION_SECRET).send().await?.json().await?;
    assert_eq!(first_receipt["runtimeBinding"]["threadId"], continued_receipt["runtimeBinding"]["threadId"]);
    assert_ne!(first_receipt["runtimeBinding"]["threadId"], reset_receipt["runtimeBinding"]["threadId"]);
    let (status, reset_view) = get(&client, &format!("/api/agent-conversations/v1/rooms/{room}/tasks/{}", reset_accepted["taskId"].as_str().unwrap()), &token).await?;
    assert_eq!(status, reqwest::StatusCode::OK);
    assert_eq!(reset_view["state"], "completed");
    assert_eq!(reset_view["effectiveSettings"]["model"], "model-a");
    assert_eq!(reset_view["usage"]["lastTotalTokens"], 40);
    let denied = turn(room, "@codex-agent Use the requested model.",
        json!({"mode":"continue","id":reset_accepted["conversationId"],"generation":reset_accepted["generation"]}),
        Some(json!({"model":"model-b","reasoningEffort":"effort-high","catalogRevision":"catalog-1"})));
    worker.kill().await?;
    worker.wait().await?;
    worker = spawn_worker(&client, "provider_denied").await?;
    let (status, denied_accepted) = post_json(&client, "/api/agent-conversations/v1/turns", &token, &denied).await?;
    assert_eq!(status, reqwest::StatusCode::ACCEPTED, "{denied_accepted}");
    run_mediator(false).await?;
    let (_, denied_view) = get(&client, &format!("/api/agent-conversations/v1/rooms/{room}/tasks/{}", denied_accepted["taskId"].as_str().unwrap()), &token).await?;
    assert_eq!(denied_view["state"], "failed", "{denied_view}");
    assert!(denied_view["replyEventId"].is_null());
    let records = native_records(&state().join("native-requests.jsonl"))?;
    let denied_turn = records.iter().rev().find(|r| r["request"]["method"] == "turn/start").unwrap();
    assert_eq!(denied_turn["request"]["params"]["model"], "gpt-6-sol");
    assert_eq!(records.iter().filter(|r| r["request"]["method"] == "turn/start").count(), 4);
    assert_eq!(events_after(&pool, room, 0).await?.into_iter().filter(|e| e.actor_id == CODEX_AGENT_ID).count(), 3);
    worker_boundary(&client, &pool, &issuer, &mut worker, "hold_send", None, false, false).await?;
    worker_boundary(&client, &pool, &issuer, &mut worker, "hold_thread_start", Some("reserved"), false, false).await?;
    worker_boundary(&client, &pool, &issuer, &mut worker, "hold_turn_start", Some("running"), true, false).await?;
    worker_boundary(&client, &pool, &issuer, &mut worker, "hold_after_turn_start", Some("running"), true, true).await?;
    mediator_boundary(&client, &pool, &issuer, &gate, GatePhase::BeforeCommit).await?;
    mediator_boundary(&client, &pool, &issuer, &gate, GatePhase::AfterCommit).await?;
    worker.kill().await?;
    worker.wait().await?;
    broker_task.abort();
    catalog_task.abort();
    pool.close().await;
    let all_native = native_records(&state().join("native-requests.jsonl"))?.iter()
        .filter(|r| r["request"]["method"] == "turn/start").count();
    assert_eq!(all_native, 8);
    println!("Task9 composed fake PASS: concurrent duplicate=one task, baseline/delta IDs, secret and room isolation, reset distinct thread/defaults, retained thread after restart, model/effort switch, latest usage, denied model no fallback, six crash/commit boundaries; native turns={all_native}");
    Ok(())
}
#[tokio::main]
async fn main() -> Result<()> {
    match env::args().nth(1).as_deref() {
        Some("--worker-child") => worker_child().await,
        Some("--mediator-once") => mediator_once().await,
        _ => parent().await,
    }
}
