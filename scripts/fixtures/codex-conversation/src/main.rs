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
    time::{Duration, Instant},
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
use tokio::{io::AsyncReadExt, net::TcpListener, process::{Child, Command}, time::sleep};
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
fn assert_native_packet(record: &Value, packet: &Value) {
    let input = record["request"]["params"]["input"].as_array().expect("native input array");
    assert_eq!(input.len(), 1, "one native input item");
    assert_eq!(input[0]["type"], "text");
    assert_eq!(input[0]["text_elements"], json!([]));
    let decoded: Value = serde_json::from_str(input[0]["text"].as_str().expect("native JSON text")).expect("native input JSON");
    assert_eq!(decoded, json!({"triggerEventId":packet["triggerEventId"], "context":packet["context"]}), "native source/trigger/context identity");
    let entries = decoded["context"]["entries"].as_array().expect("native entries");
    assert_eq!(entries.iter().filter(|e| e["eventId"] == packet["triggerEventId"]).count(), 1, "native trigger occurs once");
}
fn assert_native_input(record: &Value, packet: &Value) {
    assert_native_packet(record, packet);
    let entries = packet["context"]["entries"].as_array().expect("native entries");
    assert!(entries.iter().all(|e| e["authorRole"] == "human"), "accepted reply cannot be reinjected as assistant history");
}

// Test-only storage boundary helper: marker observation is not durable receipt evidence.
async fn bound_receipt(sqlite: &sqlx::SqlitePool, task: &str) -> Result<sqlx::sqlite::SqliteRow> {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(row) = sqlx::query("SELECT phase,thread_id,turn_id FROM receipts WHERE task_id=?")
            .bind(task).fetch_optional(sqlite).await? {
            if row.get::<String,_>("phase") == "running"
                && row.get::<Option<String>,_>("thread_id").is_some_and(|id| !id.is_empty())
                && row.get::<Option<String>,_>("turn_id").as_deref() == Some("turn-exact") { return Ok(row); }
        }
        if Instant::now() >= deadline { return Err("Task9 durable turn binding checkpoint timed out".into()); }
        sleep(Duration::from_millis(10)).await;
    }
}

async fn run_mediator() -> Result<()> {
    let mut child = Command::new(env::current_exe()?).arg("--mediator-once")
        .stderr(std::process::Stdio::piped()).kill_on_drop(true).spawn()?;
    let mut diagnostic = Vec::new();
    child.stderr.take().unwrap().take(8193).read_to_end(&mut diagnostic).await?;
    if diagnostic.len() > 8192 {
        stop_child(&mut child).await?;
        return Err("Task9 mediator diagnostic exceeded synthetic capture bound".into());
    }
    let status = child.wait().await?;
    if !status.success() || !diagnostic.is_empty() {
        return Err(format!("Task9 mediator exited {status}; bounded synthetic stderr: {}",
            String::from_utf8_lossy(&diagnostic)).into());
    }
    Ok(())
}
struct NativeGroup(Option<i32>);
impl NativeGroup {
    fn stop(&mut self) -> Result<()> {
        if let Some(pid) = self.0 {
            if unsafe { libc::kill(-pid, libc::SIGKILL) } != 0
                && std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
                return Err(std::io::Error::last_os_error().into());
            }
            self.0 = None;
        }
        Ok(())
    }
}
impl Drop for NativeGroup {
    fn drop(&mut self) {
        if let Err(error) = self.stop() { eprintln!("Task9 native-group cleanup failed: {error}"); }
    }
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
    // Register the native group before any fallible receipt/projection assertion.
    // The outer owner also scans exact executable/capture identities on any exit.
    let mut native = NativeGroup(if scenario == "hold_send" { None } else {
        let checkpoint: Value = serde_json::from_slice(&std::fs::read(&marker)?)?;
        Some(checkpoint["pid"].as_i64().ok_or("Task9 native checkpoint has no PID")?.try_into()?)
    });
    assert_ne!(env::var("TASK9_INJECT_FAILURE").as_deref(), Ok(scenario), "Task9 injected boundary assertion failure");
    let mediator_record: Value = serde_json::from_slice(&std::fs::read(state().join("mediator").join(format!("{}.json", accepted["taskId"].as_str().unwrap())))?)?;
    assert_eq!(mediator_record["phase"], "submission-intent", "{scenario}");
    let sqlite = sqlx::SqlitePool::connect(&format!("sqlite://{}?mode=rw", state().join("receipts.sqlite").display())).await?;
    let task = accepted["taskId"].as_str().unwrap();
    let receipt = if expect_turn { Some(bound_receipt(&sqlite, task).await?) } else {
        sqlx::query("SELECT phase,thread_id,turn_id FROM receipts WHERE task_id=?")
            .bind(task).fetch_optional(&sqlite).await?
    };
    if let Some(expected_phase) = expected_phase {
        let receipt = receipt.expect("Task9 worker receipt must exist");
        assert_eq!(receipt.get::<String,_>("phase"), expected_phase, "{scenario}");
        assert_eq!(receipt.get::<Option<String>,_>("thread_id").is_some(), expect_thread, "{scenario}");
        assert_eq!(receipt.get::<Option<String>,_>("turn_id").is_some(), expect_turn, "{scenario}");
    } else { assert!(receipt.is_none(), "{scenario} reached worker receipt before submission"); }
    sqlite.close().await;
    stop_child(&mut mediator).await?;
    stop_child(worker).await?;
    native.stop()?;
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
    run_mediator().await?;
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
    run_mediator().await?;
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
async fn worker_receipt(client: &Client, task: &str) -> Result<Value> {
    Ok(client.get(format!("http://127.0.0.1:9091/control/v1/receipts/{task}"))
        .bearer_auth(INVOCATION_SECRET).send().await?.error_for_status()?.json().await?)
}
fn native_turn_count() -> Result<usize> {
    Ok(native_records(&state().join("native-requests.jsonl"))?.iter()
        .filter(|r| r["request"]["method"] == "turn/start").count())
}
async fn rejected_completion_recovery(
    client: &Client, pool: &PgPool, issuer: &Issuer, gate: &UpdateGate, worker: &mut Child,
) -> Result<()> {
    let room = Uuid::new_v4();
    let token = issuer.token(Uuid::new_v4().to_string(), Some("human"), Some("Maya"));
    let before = native_turn_count()?;
    let (status, old) = post_json(client, "/api/agent-conversations/v1/turns", &token,
        &turn(room, "@codex-agent Task9 completion rejected by broker.", json!({"mode":"new"}), None)).await?;
    assert_eq!(status, reqwest::StatusCode::ACCEPTED, "{old}");
    let task = old["taskId"].as_str().unwrap();
    gate.arm(task, GatePhase::BeforeCommit).await;
    let mut mediator = Command::new(env::current_exe()?).arg("--mediator-once").kill_on_drop(true).spawn()?;
    wait_for_file(&gate.marker(task)).await?;
    let completed = worker_receipt(client, task).await?;
    assert_eq!(completed["phase"], "completed");
    assert!(completed["acknowledgement"].is_null());
    // The broker still owns an active task; fresh New cannot supersede it.
    let (busy, _) = post_json(client, "/api/agent-conversations/v1/turns", &token,
        &turn(room, "@codex-agent Task9 premature New.", json!({"mode":"new"}), None)).await?;
    assert_eq!(busy, reqwest::StatusCode::CONFLICT);
    stop_child(&mut mediator).await?;
    let record_path = state().join("mediator").join(format!("{task}.json"));
    let record: Value = serde_json::from_slice(&std::fs::read(&record_path)?)?;
    let expires_at: chrono::DateTime<Utc> = sqlx::query_scalar("SELECT expires_at FROM conversation_tasks WHERE task_id=$1")
        .bind(Uuid::parse_str(task)?).fetch_one(pool).await?;
    sleep((expires_at - Utc::now()).to_std().unwrap_or_default() + Duration::from_millis(100)).await;
    // Submit the actual persisted completion through the authenticated broker API
    // after expiry. Broker transaction rejects it and stores a terminal failure.
    let rejected = client.post(format!("http://127.0.0.1:8080/internal/agent-conversations/v1/tasks/{task}/updates"))
        .bearer_auth(issuer.token("service-account-thought-khoral-agent-gateway".into(), None, None))
        .header("x-thought-khoral-lease-token", record["lease"].as_str().unwrap())
        .json(&record["update"]).send().await?;
    assert!(!rejected.status().is_success());
    let failure: Value = rejected.json().await?;
    assert_eq!(failure["code"], "timeout", "{failure}");
    run_mediator().await?;
    let quarantined: Value = serde_json::from_slice(&std::fs::read(&record_path)?)?;
    assert_eq!(quarantined["phase"], "quarantined");
    assert_eq!(worker_receipt(client, task).await?, completed, "quarantine must preserve completed receipt");
    let (_, failed) = get(client, &format!("/api/agent-conversations/v1/rooms/{room}/tasks/{task}"), &token).await?;
    assert_eq!(failed["state"], "failed");
    assert!(failed["replyEventId"].is_null());
    let stale = turn(room, "@codex-agent Task9 stale continuation.", json!({"mode":"continue","id":old["conversationId"],"generation":old["generation"]}), None);
    assert!(!post_json(client, "/api/agent-conversations/v1/turns", &token, &stale).await?.0.is_success());
    stop_child(worker).await?;
    *worker = spawn_worker(client, "usage").await?;
    let (status, fresh) = post_json(client, "/api/agent-conversations/v1/turns", &token,
        &turn(room, "@codex-agent Task9 explicit fresh recovery.", json!({"mode":"new"}), None)).await?;
    assert_eq!(status, reqwest::StatusCode::ACCEPTED, "{fresh}");
    assert!(fresh["generation"].as_u64().unwrap() > old["generation"].as_u64().unwrap());
    assert_ne!(fresh["conversationId"], old["conversationId"]);
    run_mediator().await?;
    let fresh_task = fresh["taskId"].as_str().unwrap();
    let (_, fresh_view) = get(client, &format!("/api/agent-conversations/v1/rooms/{room}/tasks/{fresh_task}"), &token).await?;
    assert_eq!(fresh_view["state"], "completed", "{fresh_view}");
    let fresh_receipt = worker_receipt(client, fresh_task).await?;
    assert_ne!(fresh_receipt["runtimeBinding"]["threadId"], completed["runtimeBinding"]["threadId"]);
    assert_eq!(worker_receipt(client, task).await?, completed);
    assert!(!post_json(client, "/api/agent-conversations/v1/turns", &token, &stale).await?.0.is_success());
    assert_eq!(native_turn_count()? - before, 2);
    assert_eq!(events_after(pool, room, 0).await?.iter().filter(|e| e.actor_id == CODEX_AGENT_ID).count(), 1);
    println!("Task9 rejected completion recovery: broker timeout, quarantine, fresh higher generation/distinct thread after restart; two native turns, one public reply");

    // A genuine failed worker resume/receipt preserves session_unavailable.
    let thread = fresh_receipt["runtimeBinding"]["threadId"].as_str().unwrap();
    assert!(thread.starts_with("thread-") && thread.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'));
    std::fs::remove_file(state().join("native/sessions").join(format!("{thread}.jsonl")))?;
    let (status, missing) = post_json(client, "/api/agent-conversations/v1/turns", &token,
        &turn(room, "@codex-agent Task9 missing native history.", json!({"mode":"continue","id":fresh["conversationId"],"generation":fresh["generation"]}), None)).await?;
    assert_eq!(status, reqwest::StatusCode::ACCEPTED);
    assert_safe_failure(client, pool, room, &token, &missing, "session_unavailable", 1).await?;
    assert_eq!(native_turn_count()? - before, 2, "missing history must not start a replacement turn");
    Ok(())
}
async fn assert_safe_failure(client: &Client, pool: &PgPool, room: Uuid, token: &str, accepted: &Value, code: &str, replies: usize) -> Result<()> {
    run_mediator().await?;
    let task = accepted["taskId"].as_str().unwrap();
    let receipt = worker_receipt(client, task).await?;
    assert_eq!(receipt["phase"], "failed", "{receipt}");
    assert_eq!(receipt["error"], code);
    let (_, view) = get(client, &format!("/api/agent-conversations/v1/rooms/{room}/tasks/{task}"), token).await?;
    assert_eq!(view["state"], "failed", "{view}");
    assert_eq!(view["failure"]["code"], code, "{view}");
    assert!(view["replyEventId"].is_null());
    assert!(!view.to_string().contains("synthetic missing native file"));
    assert_eq!(events_after(pool, room, 0).await?.iter().filter(|e| e.actor_id == CODEX_AGENT_ID).count(), replies);
    println!("Task9 receipt-correlated safe failure: {code}; no additional public reply");
    Ok(())
}
async fn unavailable_runtime(client: &Client, pool: &PgPool, issuer: &Issuer, worker: &mut Child) -> Result<()> {
    stop_child(worker).await?;
    *worker = spawn_worker(client, "runtime_unavailable").await?;
    let room = Uuid::new_v4();
    let token = issuer.token(Uuid::new_v4().to_string(), Some("human"), Some("Leo"));
    let before = native_turn_count()?;
    let (status, accepted) = post_json(client, "/api/agent-conversations/v1/turns", &token,
        &turn(room, "@codex-agent Task9 unavailable native runtime.", json!({"mode":"new"}), None)).await?;
    assert_eq!(status, reqwest::StatusCode::ACCEPTED);
    assert_safe_failure(client, pool, room, &token, &accepted, "runtime_unavailable", 0).await?;
    assert_eq!(native_turn_count()?, before);
    Ok(())
}
// Candidate-only synthetic UI/HTTP acceptance; no provider or browser claim.
fn exact_keys(value: &Value, expected: &[&str]) {
    let mut actual = value.as_object().expect("closed evidence object").keys().map(String::as_str).collect::<Vec<_>>();
    let mut expected = expected.to_vec(); actual.sort(); expected.sort(); assert_eq!(actual, expected);
}
fn selected(model: &str, effort: &str) -> Value {
    json!({"model":model,"reasoningEffort":effort,"catalogRevision":"catalog-1"})
}
fn assert_ui_pair(evidence: &Value, phase: &str, room: Uuid, capabilities: (bool, bool), expected: &Value, task: &str) {
    exact_keys(evidence, &["phase", "capabilities", "displayedSettings", "submittedSettings", "acceptedTurn", "defaultsReads", "catalogReads"]);
    assert_eq!(evidence["phase"], phase);
    assert_eq!(evidence["capabilities"], json!({"modelSelection":capabilities.0,"reasoningEffort":capabilities.1}));
    let accepted = &evidence["acceptedTurn"];
    assert_eq!(accepted["taskId"], task); assert_eq!(accepted["roomId"], room.to_string());
    assert_eq!(accepted["selectedSettings"], *expected);
    if capabilities.0 || capabilities.1 {
        assert_eq!(evidence["displayedSettings"], *expected);
        assert_eq!(evidence["displayedSettings"], evidence["submittedSettings"]);
        assert_eq!(evidence["submittedSettings"], accepted["selectedSettings"]);
        assert_eq!(evidence["defaultsReads"], if phase == "restored" { 0 } else { 1 });
        // New first restores the current catalog and then explicitly discovers again.
        assert_eq!(evidence["catalogReads"], if phase == "new" { 2 } else { 1 });
    } else {
        assert!(evidence["displayedSettings"].is_null()); assert!(evidence["submittedSettings"].is_null());
        assert_eq!(evidence["defaultsReads"], 0); assert_eq!(evidence["catalogReads"], 0);
    }
}
fn assert_native_pair(record: &Value, expected: &Value, thread: &Value) {
    assert_eq!(record["request"]["params"]["threadId"], *thread);
    let model = match expected["model"].as_str().unwrap() { "model-a" => "gpt-6.1-sol", "model-b" => "gpt-6-sol", _ => panic!("unexpected synthetic model") };
    let effort = match expected["reasoningEffort"].as_str().unwrap() { "effort-medium" => "medium", "effort-high" => "high", _ => panic!("unexpected synthetic effort") };
    assert_eq!(record["request"]["params"]["model"], model); assert_eq!(record["request"]["params"]["effort"], effort);
}
fn write_evidence(path: &Path, value: &Value) -> Result<()> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(path)?;
    writeln!(file, "{}", serde_json::to_string_pretty(value)?)?;
    Ok(())
}
async fn matrix_policy(gateway: &GatewayState, client: &Client, mode: &str) -> Result<()> {
    let mut value = policy(); value.turn_deadline_seconds = 60;
    if mode == "b" { value.model = "model-b".into(); value.reasoning_effort = "effort-high".into(); }
    if mode == "stale" { value.catalog_revision = "catalog-2".into(); }
    if mode == "removed" { value.models.retain(|model| model.id != "model-b"); }
    let catalog = if mode == "unavailable" { None } else { Some(Arc::new(LoopbackCatalog(client.clone())) as Arc<dyn CatalogQuery>) };
    gateway.configure_conversations(value, catalog).await?;
    Ok(())
}
async fn room_counts(pool: &PgPool, room: Uuid) -> Result<Value> {
    let mut counts = serde_json::Map::new();
    // Fixed fixture-only table names, values remain parameter-bound.
    for table in ["agent_conversations", "conversation_tasks", "room_events", "room_requests", "conversation_updates", "conversation_disclosures"] {
        let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table} WHERE room_id=$1")).bind(room).fetch_one(pool).await?;
        counts.insert(table.into(), json!(count));
    }
    Ok(Value::Object(counts))
}
async fn defaults_read(client: &Client, pool: &PgPool, room: Uuid, token: &str) -> Result<Value> {
    let before = room_counts(pool, room).await?;
    assert!(before.as_object().unwrap().values().all(|value| *value == 0));
    let native_before = native_turn_count()?;
    let records_before = native_records(&state().join("native-requests.jsonl"))?;
    let threads_before = records_before.iter().filter(|r| r["request"]["method"] == "thread/start").count();
    let path = format!("http://127.0.0.1:8080/api/agent-conversations/v1/rooms/{room}/agents/{CODEX_AGENT_ID}/defaults");
    for auth in [false, true] {
        let mut request = client.get(&path);
        if auth { request = request.bearer_auth(token); }
        let response = request.send().await?;
        assert_eq!(response.status(), if auth { reqwest::StatusCode::OK } else { reqwest::StatusCode::UNAUTHORIZED });
        assert_eq!(response.headers()["cache-control"], "no-store");
        let view: Value = response.json().await?;
        if auth {
            exact_keys(&view, &["profileVersion", "roomId", "agentId", "selectedSettings"]);
            assert_eq!(view["profileVersion"], PROFILE_VERSION); assert_eq!(view["roomId"], room.to_string());
            assert_eq!(view["agentId"], CODEX_AGENT_ID.to_string()); assert_eq!(view["selectedSettings"], selected("model-b", "effort-high"));
        } else {
            exact_keys(&view, &["profileVersion", "requestId", "code", "message"]);
            assert_eq!(view["code"], "authentication_required");
        }
    }
    assert_eq!(room_counts(pool, room).await?, before); assert_eq!(native_turn_count()?, native_before);
    assert_eq!(native_records(&state().join("native-requests.jsonl"))?.iter().filter(|r| r["request"]["method"] == "thread/start").count(), threads_before);
    Ok(json!({"roomId":room,"countsBefore":before,"countsAfter":before,"nativeTurnDelta":0,"nativeThreadDelta":0,"authenticated":true,"unauthenticatedRejected":true,"noStore":true,"closedResponse":true}))
}
async fn ui_child(client: &Client, gateway: &GatewayState, room: Uuid, token: &str, phase: &str, capabilities: (bool, bool), mutation: Option<&str>) -> Result<Value> {
    let label = format!("{}-{}-{phase}", room, if capabilities.0 { "model" } else { "no-model" });
    let evidence = state().join(format!("ui-{label}.json"));
    let display = state().join(format!("display-{label}.json"));
    let release = state().join(format!("release-{label}.json"));
    assert!(!evidence.exists() && !display.exists() && !release.exists(), "UI evidence must be fresh");
    let stdout = std::fs::OpenOptions::new().write(true).create_new(true).open(state().join(format!("ui-{label}.stdout.log")))?;
    let stderr = std::fs::OpenOptions::new().write(true).create_new(true).open(state().join(format!("ui-{label}.stderr.log")))?;
    let mut command = Command::new("npm");
    command.args(["test", "--", "src/features/room/CodexDefaults.composed.test.tsx"])
        .current_dir(env::var("TASK9_UI_REPO")?).env("RUN_CODEX_DEFAULTS_COMPOSED", "1")
        .env("CODEX_DEFAULTS_BROKER_ORIGIN", "http://127.0.0.1:8080")
        .env("CODEX_DEFAULTS_ROOM_ID", room.to_string()).env("CODEX_DEFAULTS_TOKEN", token)
        .env("CODEX_DEFAULTS_PHASE", phase).env("CODEX_DEFAULTS_EVIDENCE_FILE", &evidence)
        .env("CODEX_DEFAULTS_MODEL_SELECTION", capabilities.0.to_string())
        .env("CODEX_DEFAULTS_REASONING_EFFORT", capabilities.1.to_string())
        .env_remove("CODEX_DEFAULTS_DISPLAY_EVIDENCE_FILE").env_remove("CODEX_DEFAULTS_SEND_RELEASE_FILE").env_remove("CODEX_DEFAULTS_EXPECT_REJECTION")
        .stdout(stdout).stderr(stderr).kill_on_drop(true);
    if let Some(mode) = mutation {
        command.env("CODEX_DEFAULTS_DISPLAY_EVIDENCE_FILE", &display).env("CODEX_DEFAULTS_SEND_RELEASE_FILE", &release);
        if mode != "default-only" { command.env("CODEX_DEFAULTS_EXPECT_REJECTION", "invalid_task_input"); }
    }
    println!("Task9 UI START {label} mutation={mutation:?}");
    let mut child = command.spawn()?;
    if let Some(mode) = mutation {
        wait_for_file(&display).await?;
        let shown: Value = serde_json::from_slice(&std::fs::read(&display)?)?;
        exact_keys(&shown, &["phase", "capabilities", "displayedSettings", "prompt"]);
        assert_eq!(shown["displayedSettings"], selected("model-b", "effort-high"));
        matrix_policy(gateway, client, if mode == "default-only" { "a" } else { mode }).await?;
        use std::io::Write;
        std::fs::OpenOptions::new().write(true).create_new(true).open(&release)?.write_all(b"{\"release\":true}\n")?;
    }
    let status = tokio::time::timeout(Duration::from_secs(45), child.wait()).await??;
    if !status.success() { return Err(format!("Task9 UI FAIL {label}: {status}; see owned UI logs").into()); }
    let value: Value = serde_json::from_slice(&std::fs::read(&evidence)?)?;
    println!("Task9 UI HTTP result {label} task={}", value["acceptedTurn"]["taskId"]);
    Ok(value)
}
async fn mediate_ui(client: &Client, pool: &PgPool, room: Uuid, token: &str, phase: &str, capabilities: (bool, bool), evidence: &Value, expected: &Value) -> Result<Value> {
    let accepted = &evidence["acceptedTurn"];
    let task = accepted["taskId"].as_str().ok_or("UI evidence has no accepted task")?;
    let frozen_row = sqlx::query("SELECT frozen_input,accepted_turn FROM conversation_tasks WHERE task_id=$1 AND room_id=$2")
        .bind(Uuid::parse_str(task)?).bind(room).fetch_one(pool).await?;
    let frozen: Value = frozen_row.get("frozen_input"); let stored: Value = frozen_row.get("accepted_turn");
    assert_eq!(*accepted, stored);
    assert_ui_pair(evidence, phase, room, capabilities, expected, frozen["taskId"].as_str().unwrap());
    let frozen_settings = json!({"model":frozen["model"],"reasoningEffort":frozen["reasoningEffort"],"catalogRevision":frozen["catalogRevision"]});
    assert_eq!(frozen["agentId"], CODEX_AGENT_ID.to_string());
    assert_eq!(frozen_settings, *expected);
    let before = native_turn_count()?; run_mediator().await?;
    assert_eq!(native_turn_count()?, before + 1);
    let (status, view) = get(client, &format!("/api/agent-conversations/v1/rooms/{room}/tasks/{task}"), token).await?;
    assert_eq!(status, reqwest::StatusCode::OK); assert_eq!(view["state"], "completed", "{view}");
    assert_eq!(view["selectedSettings"], *expected); assert_eq!(view["effectiveSettings"]["model"], expected["model"]); assert_eq!(view["effectiveSettings"]["reasoningEffort"], expected["reasoningEffort"]);
    let receipt = worker_receipt(client, task).await?;
    let thread = &receipt["runtimeBinding"]["threadId"]; assert!(thread.as_str().is_some_and(|s| !s.is_empty()));
    let records = native_records(&state().join("native-requests.jsonl"))?;
    let matching = records.iter().filter(|record| {
        record["request"]["method"] == "turn/start" && record["request"]["params"]["input"][0]["text"].as_str()
            .and_then(|text| serde_json::from_str::<Value>(text).ok()).is_some_and(|input| input["triggerEventId"] == frozen["triggerEventId"])
    }).collect::<Vec<_>>();
    assert_eq!(matching.len(), 1, "one exact task/trigger-correlated native capture");
    let native = matching[0];
    if phase == "new" {
        // A new generation's baseline includes public replies from prior threads;
        // the profile forbids substituting native bindings in any baseline.
        assert_native_packet(native, &frozen);
        assert_eq!(frozen["context"]["kind"], "baseline"); assert_eq!(frozen["context"]["baseRevision"], 0);
        assert_eq!(frozen["context"]["nativeReplyBindings"], json!([]));
        let entries = frozen["context"]["entries"].as_array().unwrap();
        let public = events_after(pool, room, 0).await?.into_iter().filter(|event| event.sequence <= frozen["context"]["revision"].as_i64().unwrap()).collect::<Vec<_>>();
        assert_eq!(entries.len(), 5); assert_eq!(entries.len(), public.len());
        assert_eq!(entries.iter().filter(|entry| entry["authorRole"] == "agent").count(), 2);
        for (entry, event) in entries.iter().zip(public.iter()) {
            assert_eq!(entry["eventId"], event.event_id.to_string()); assert_eq!(entry["text"], event.payload["text"]);
            assert_eq!(entry["authorId"], event.actor_id.to_string());
        }
    } else { assert_native_input(native, &frozen); }
    assert_native_pair(native, expected, thread);
    assert_eq!(receipt["taskId"], task);
    let result = json!({"phase":phase,"capabilities":evidence["capabilities"],"roomId":room,"taskId":task,"conversationId":accepted["conversationId"],"generation":accepted["generation"],
        "displayedSettings":evidence["displayedSettings"],"submittedSettings":evidence["submittedSettings"],"acceptedSettings":expected,"frozenSettings":frozen_settings,"effectiveSettings":view["effectiveSettings"],
        "nativeModel":native["request"]["params"]["model"],"nativeEffort":native["request"]["params"]["effort"],"threadId":thread,"defaultsReads":evidence["defaultsReads"],"catalogReads":evidence["catalogReads"],"gate":"passed"});
    write_evidence(&state().join(format!("comparison-{task}.json")), &result)?;
    println!("Task9 defaults comparison PASS: {result}");
    Ok(result)
}
async fn defaults_ui_matrix(client: &Client, pool: &PgPool, issuer: &Issuer, gateway: &GatewayState) -> Result<()> {
    let start = native_turn_count()?; let mut results = vec![]; let mut reads = vec![];
    for capabilities in [(true,true),(false,true),(true,false),(false,false)] {
        matrix_policy(gateway, client, "b").await?;
        let room = Uuid::new_v4(); let token = issuer.token(Uuid::new_v4().to_string(), Some("human"), Some("Maya"));
        reads.push(defaults_read(client, pool, room, &token).await?);
        let mut previous: Option<Value> = None;
        for phase in ["initial", "restored", "new"] {
            if phase == "restored" { matrix_policy(gateway, client, "a").await?; }
            let evidence = ui_child(client, gateway, room, &token, phase, capabilities, None).await?;
            let pair = if phase == "new" { selected("model-a", "effort-medium") } else { selected("model-b", "effort-high") };
            let result = mediate_ui(client, pool, room, &token, phase, capabilities, &evidence, &pair).await?;
            if let Some(prior) = &previous {
                if phase == "new" { assert_ne!(prior["threadId"], result["threadId"]); assert_ne!(prior["conversationId"], result["conversationId"]); }
                else { assert_eq!(prior["threadId"], result["threadId"]); assert_eq!(prior["conversationId"], result["conversationId"]); }
            }
            previous = Some(result.clone()); results.push(result);
        }
    }
    assert_eq!(results.len(), 12); assert_eq!(native_turn_count()? - start, 12);
    let mut mutations = vec![];
    for mode in ["default-only", "stale", "removed"] {
        matrix_policy(gateway, client, "b").await?;
        let room = Uuid::new_v4(); let token = issuer.token(Uuid::new_v4().to_string(), Some("human"), Some("Leo"));
        let before = room_counts(pool, room).await?; let native_before = native_turn_count()?;
        let evidence = ui_child(client, gateway, room, &token, "initial", (true,true), Some(mode)).await?;
        if mode == "default-only" {
            let result = mediate_ui(client, pool, room, &token, "initial", (true,true), &evidence, &selected("model-b", "effort-high")).await?;
            let after = room_counts(pool, room).await?; assert_eq!(after["conversation_tasks"], 1); assert_eq!(after["agent_conversations"], 1);
            // Recover the original request from immutable public request storage, not a newly invented request.
            let task = evidence["acceptedTurn"]["taskId"].as_str().unwrap();
            let request: Value = sqlx::query_scalar("SELECT request_fingerprint->'intent' FROM room_requests WHERE room_id=$1 AND request_id=(SELECT request_id FROM conversation_tasks WHERE task_id=$2)")
                .bind(room).bind(Uuid::parse_str(task)?).fetch_one(pool).await?;
            matrix_policy(gateway, client, "unavailable").await?;
            let (unavailable, _) = get(client, &format!("/api/agent-conversations/v1/rooms/{room}/agents/{CODEX_AGENT_ID}/defaults"), &token).await?;
            assert_eq!(unavailable, reqwest::StatusCode::SERVICE_UNAVAILABLE);
            let native_before_replay = native_turn_count()?;
            let (status, replay) = post_json(client, "/api/agent-conversations/v1/turns", &token, &request).await?;
            assert_eq!(status, reqwest::StatusCode::ACCEPTED); assert_eq!(replay, evidence["acceptedTurn"]);
            run_mediator().await?; assert_eq!(native_turn_count()?, native_before_replay); assert_eq!(room_counts(pool, room).await?, after);
            mutations.push(json!({"mode":mode,"comparison":result,"replayImmutable":true,"replayNativeTurnDelta":0}));
        } else {
            exact_keys(&evidence, &["phase", "capabilities", "displayedSettings", "submittedSettings", "rejection", "promptRetained", "refreshRequired", "sendBlocked", "defaultsReads", "catalogReads"]);
            assert_eq!(evidence["displayedSettings"], selected("model-b", "effort-high")); assert_eq!(evidence["submittedSettings"], evidence["displayedSettings"]);
            assert_eq!(evidence["rejection"]["profileError"]["code"], "invalid_task_input");
            for key in ["promptRetained", "refreshRequired", "sendBlocked"] { assert_eq!(evidence[key], true); }
            assert_eq!(evidence["defaultsReads"], 1); assert_eq!(evidence["catalogReads"], 1);
            run_mediator().await?;
            assert_eq!(room_counts(pool, room).await?, before); assert_eq!(native_turn_count()?, native_before);
            mutations.push(json!({"mode":mode,"uiEvidence":evidence,"roomId":room,"taskId":null,"countsBefore":before,"countsAfter":before,"nativeTurnDelta":0,"gate":"passed"}));
        }
        println!("Task9 defaults mutation PASS: {}", mutations.last().unwrap());
    }
    assert_eq!(native_turn_count()? - start, 13);
    write_evidence(&state().join("defaults-matrix.json"), &json!({"evidenceClass":"local-synthetic-real-http-jsdom-fake-native","gate":"passed","lifecycle":results,"readOnly":reads,"mutations":mutations,"lifecycleCount":12,"additionalNativeTurns":13,"retainedBaselineNativeTurns":11,"retainedBoundaryScenarios":6}))?;
    println!("Task9 defaults candidate PASS: 12 actual UI lifecycle phases, 4 read-only auth/no-store checks, 3 display/mutate/send cases, immutable unavailable replay; additional native turns=13, baseline=11");
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
    let broker_router = app(gateway_state.clone()).merge(token_route).layer(middleware::from_fn_with_state(gate.clone(), gate_update));
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
    run_mediator().await?;
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
    assert_native_input(first_turns[0], &frozen);
    assert_eq!(frozen["context"]["nativeReplyBindings"], json!([]));
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
    assert_eq!(delta["context"]["entries"].as_array().unwrap().len(), 2, "only intervening and current trigger entries");
    assert_eq!(delta["triggerEventId"], continued["triggerEventId"]);
    assert_eq!(delta["context"]["entries"][0]["eventId"], json!(intervening));
    assert_eq!(delta["context"]["entries"][1]["eventId"], continued["triggerEventId"]);
    assert_eq!(delta["context"]["entries"][0]["text"], "Task9 intervening discussion: budget is twelve.");
    assert_eq!(delta["context"]["entries"][1]["text"], "@codex-agent What changed?");
    assert_eq!(delta["context"]["nativeReplyBindings"], json!([{
        "eventId":view["replyEventId"], "sequence":events.last().unwrap().sequence,
        "sourceTaskId":accepted["taskId"], "generation":accepted["generation"],
        // SHA-256 of the known synthetic final text, independently fixed by this fixture.
        "textDigest":"228c39c3d786e5a4c8018cc5d7ca5f272cd0985c5f60cdc8e36c6fd769d04021"
    }]));
    for excluded in ["violet", "orange", "Task9 Maya proposed green.", "Task9 Leo corrected the timing to Friday.", "@codex-agent What did Maya propose?", "Maya proposed green."] {
        assert!(!delta.to_string().contains(excluded), "continuation reinjected excluded/old text");
    }
    let wrong = turn(other_room, "@codex-agent Cross-room bind?", json!({"mode":"continue","id":accepted["conversationId"],"generation":accepted["generation"]}), None);
    let (wrong_status, _) = post_json(&client, "/api/agent-conversations/v1/turns", &token, &wrong).await?;
    assert!(!wrong_status.is_success(), "another room bound the conversation");
    worker.kill().await?;
    worker.wait().await?;
    worker = spawn_worker(&client, "usage").await?;
    run_mediator().await?;
    let (status, second_view) = get(&client, &format!("/api/agent-conversations/v1/rooms/{room}/tasks/{}", continued["taskId"].as_str().unwrap()), &token).await?;
    assert_eq!(status, reqwest::StatusCode::OK);
    assert_eq!(second_view["state"], "completed", "{second_view}");
    let records = native_records(&state().join("native-requests.jsonl"))?;
    let continued_turns = records.iter().filter(|r| r["request"]["method"] == "turn/start").collect::<Vec<_>>();
    assert_eq!(continued_turns.len(), 2);
    assert_native_input(continued_turns[0], &frozen);
    assert_native_input(continued_turns[1], &delta);
    let second_prompt = continued_turns[1]["request"]["params"]["input"][0]["text"].as_str().unwrap();
    assert_eq!(second_prompt.matches("@codex-agent What changed?").count(), 1);
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

    // Another authenticated human reloads the worker and omits overrides.
    // The accepted shared pair must survive through native execution.
    stop_child(&mut worker).await?;
    worker = spawn_worker(&client, "usage").await?;
    let maya_token = issuer.token(maya.to_string(), Some("human"), Some("Maya"));
    let (status, shared) = post_json(&client, "/api/agent-conversations/v1/turns", &maya_token,
        &turn(room, "@codex-agent Task9 keep our shared settings.", json!({"mode":"continue","id":accepted["conversationId"],"generation":accepted["generation"]}), None)).await?;
    assert_eq!(status, reqwest::StatusCode::ACCEPTED, "{shared}");
    assert_eq!(shared["selectedSettings"]["model"], "model-b");
    assert_eq!(shared["selectedSettings"]["reasoningEffort"], "effort-high");
    run_mediator().await?;
    let (_, shared_view) = get(&client, &format!("/api/agent-conversations/v1/rooms/{room}/tasks/{}", shared["taskId"].as_str().unwrap()), &maya_token).await?;
    assert_eq!(shared_view["state"], "completed", "{shared_view}");
    assert_eq!(shared_view["effectiveSettings"]["model"], "model-b");
    assert_eq!(shared_view["effectiveSettings"]["reasoningEffort"], "effort-high");
    let records = native_records(&state().join("native-requests.jsonl"))?;
    let shared_turn = records.iter().rev().find(|r| r["request"]["method"] == "turn/start").unwrap();
    assert_eq!(shared_turn["request"]["params"]["model"], "gpt-6-sol");
    assert_eq!(shared_turn["request"]["params"]["effort"], "high");
    println!("Task9 omitted continuation: another human after restart retains selected/effective/native model-b/high");

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
    run_mediator().await?;
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
    run_mediator().await?;
    let (_, denied_view) = get(&client, &format!("/api/agent-conversations/v1/rooms/{room}/tasks/{}", denied_accepted["taskId"].as_str().unwrap()), &token).await?;
    assert_eq!(denied_view["state"], "failed", "{denied_view}");
    assert!(denied_view["replyEventId"].is_null());
    assert_eq!(denied_view["failure"]["code"], "execution_failed");
    assert!(!denied_view.to_string().contains("synthetic provider denial"));
    assert_eq!(worker_receipt(&client, denied_accepted["taskId"].as_str().unwrap()).await?["error"], "execution_failed");
    println!("Task9 synthetic provider denial: receipt-correlated execution_failed, no fallback/reply");
    let records = native_records(&state().join("native-requests.jsonl"))?;
    let denied_turn = records.iter().rev().find(|r| r["request"]["method"] == "turn/start").unwrap();
    assert_eq!(denied_turn["request"]["params"]["model"], "gpt-6-sol");
    assert_eq!(records.iter().filter(|r| r["request"]["method"] == "turn/start").count(), 5);
    assert_eq!(events_after(&pool, room, 0).await?.into_iter().filter(|e| e.actor_id == CODEX_AGENT_ID).count(), 4);
    worker_boundary(&client, &pool, &issuer, &mut worker, "hold_send", None, false, false).await?;
    worker_boundary(&client, &pool, &issuer, &mut worker, "hold_thread_start", Some("reserved"), false, false).await?;
    worker_boundary(&client, &pool, &issuer, &mut worker, "hold_turn_start", Some("running"), true, false).await?;
    worker_boundary(&client, &pool, &issuer, &mut worker, "hold_after_turn_start", Some("running"), true, true).await?;
    mediator_boundary(&client, &pool, &issuer, &gate, GatePhase::BeforeCommit).await?;
    mediator_boundary(&client, &pool, &issuer, &gate, GatePhase::AfterCommit).await?;
    rejected_completion_recovery(&client, &pool, &issuer, &gate, &mut worker).await?;
    unavailable_runtime(&client, &pool, &issuer, &mut worker).await?;
    let retained_native = native_turn_count()?;
    assert_eq!(retained_native, 11, "retained baseline native-turn assertion");
    if env::var("TASK9_DEFAULTS_CANDIDATE").as_deref() == Ok("1") {
        stop_child(&mut worker).await?;
        worker = spawn_worker(&client, "usage").await?;
        defaults_ui_matrix(&client, &pool, &issuer, &gateway_state).await?;
    }
    worker.kill().await?;
    worker.wait().await?;
    broker_task.abort();
    catalog_task.abort();
    pool.close().await;
    let all_native = native_records(&state().join("native-requests.jsonl"))?.iter()
        .filter(|r| r["request"]["method"] == "turn/start").count();
    assert_eq!(all_native, if env::var("TASK9_DEFAULTS_CANDIDATE").as_deref() == Ok("1") { 24 } else { 11 });
    println!("Task9 composed fake PASS: concurrent duplicate=one task, baseline/delta IDs, secret and room isolation, reset distinct thread/defaults, retained thread after restart, model/effort switch, latest usage, denied model no fallback, six crash/commit boundaries; rejected-completion recovery; omitted shared settings; three safe failures; native turns={all_native}");
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_input_rejects_source_changes_duplicate_trigger_old_history_and_reply_reinjection() {
        let packet = json!({"triggerEventId":"trigger-2", "context":{
            "entries":[{"eventId":"intervening", "authorRole":"human", "text":"new fact"},
                {"eventId":"trigger-2", "authorRole":"human", "text":"@codex-agent new question"}],
            "nativeReplyBindings":[{"eventId":"reply-1", "sourceTaskId":"task-1", "generation":1, "sequence":5,"textDigest":"expected-digest"}]
        }});
        let capture = |decoded: &Value| json!({"request":{"params":{"input":[{"type":"text", "text_elements":[], "text":decoded.to_string()}]}}});
        assert_native_input(&capture(&packet), &packet);
        for change in ["trigger", "source", "duplicate", "history", "reply", "binding-source", "binding-missing"] {
            let mut altered = packet.clone();
            match change {
                "trigger" => altered["triggerEventId"] = json!("wrong"),
                "source" => altered["context"]["entries"][0]["eventId"] = json!("wrong"),
                "duplicate" => {
                    let trigger = altered["context"]["entries"][1].clone();
                    altered["context"]["entries"].as_array_mut().unwrap().push(trigger);
                },
                "history" => altered["context"]["entries"].as_array_mut().unwrap().push(json!({"eventId":"old", "authorRole":"human", "text":"old fact"})),
                "reply" => altered["context"]["entries"].as_array_mut().unwrap().push(json!({"eventId":"reply-1", "authorRole":"agent", "text":"accepted answer"})),
                "binding-source" => altered["context"]["nativeReplyBindings"][0]["sourceTaskId"] = json!("wrong"),
                "binding-missing" => altered["context"]["nativeReplyBindings"] = json!([]),
                _ => unreachable!(),
            }
            assert!(std::panic::catch_unwind(|| assert_native_input(&capture(&altered), &packet)).is_err(), "missed native mutation {change}");
        }
    }
    #[test]
    fn defaults_comparison_refuses_pair_task_thread_and_read_mutations() {
        let room = Uuid::new_v4(); let settings = selected("model-b", "effort-high");
        let evidence = json!({"phase":"initial","capabilities":{"modelSelection":true,"reasoningEffort":true},"displayedSettings":settings,"submittedSettings":settings,
            "acceptedTurn":{"roomId":room,"taskId":"task-exact","selectedSettings":settings},"defaultsReads":1,"catalogReads":1});
        assert_ui_pair(&evidence, "initial", room, (true,true), &settings, "task-exact");
        for mutation in ["model", "effort", "task", "defaults", "catalog", "hidden"] {
            let mut wrong = evidence.clone();
            match mutation {
                "model" => wrong["displayedSettings"]["model"] = json!("model-a"),
                "effort" => wrong["submittedSettings"]["reasoningEffort"] = json!("effort-medium"),
                "task" => wrong["acceptedTurn"]["taskId"] = json!("wrong-task"),
                "defaults" => wrong["defaultsReads"] = json!(2),
                "catalog" => wrong["catalogReads"] = json!(0),
                "hidden" => wrong["displayedSettings"] = Value::Null,
                _ => unreachable!(),
            }
            assert!(std::panic::catch_unwind(|| assert_ui_pair(&wrong, "initial", room, (true,true), &settings, "task-exact")).is_err(), "missed {mutation}");
        }
        let baseline = json!({"triggerEventId":"trigger-new","context":{"kind":"baseline","baseRevision":0,"nativeReplyBindings":[],"entries":[
            {"eventId":"prior-public-reply","authorRole":"agent","text":"public reply"},{"eventId":"trigger-new","authorRole":"human","text":"new question"}]}});
        let capture = json!({"request":{"params":{"input":[{"type":"text","text_elements":[],"text":baseline.to_string()}]}}});
        assert_native_packet(&capture, &baseline);
        assert!(std::panic::catch_unwind(|| assert_native_input(&capture, &baseline)).is_err(), "human-only continuation protection remains strict");
        let mut changed_baseline = baseline.clone(); changed_baseline["context"]["entries"][0]["text"] = json!("invented reply");
        assert!(std::panic::catch_unwind(|| assert_native_packet(&capture, &changed_baseline)).is_err());
        let native = json!({"request":{"params":{"threadId":"thread-exact","model":"gpt-6-sol","effort":"high"}}});
        assert_native_pair(&native, &settings, &json!("thread-exact"));
        for field in ["model", "effort", "threadId"] {
            let mut wrong = native.clone(); wrong["request"]["params"][field] = json!("wrong");
            assert!(std::panic::catch_unwind(|| assert_native_pair(&wrong, &settings, &json!("thread-exact"))).is_err());
        }
        let mut hidden = evidence.clone(); hidden["capabilities"] = json!({"modelSelection":false,"reasoningEffort":false});
        hidden["displayedSettings"] = Value::Null; hidden["submittedSettings"] = Value::Null; hidden["defaultsReads"] = json!(0); hidden["catalogReads"] = json!(0);
        assert_ui_pair(&hidden, "initial", room, (false,false), &settings, "task-exact");
        hidden["defaultsReads"] = json!(1);
        assert!(std::panic::catch_unwind(|| assert_ui_pair(&hidden, "initial", room, (false,false), &settings, "task-exact")).is_err());
    }
    #[tokio::test]
    async fn durable_checkpoint_refuses_an_unbound_receipt() -> Result<()> {
        let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await?;
        sqlx::query("CREATE TABLE receipts(task_id TEXT,phase TEXT,thread_id TEXT,turn_id TEXT)").execute(&pool).await?;
        sqlx::query("INSERT INTO receipts VALUES('task','running','thread',NULL)").execute(&pool).await?;
        assert!(bound_receipt(&pool, "task").await.is_err());
        Ok(())
    }
    #[tokio::test]
    async fn turn_boundary_waits_for_delayed_durable_binding() -> Result<()> {
        let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await?;
        sqlx::query("CREATE TABLE receipts(task_id TEXT,phase TEXT,thread_id TEXT,turn_id TEXT)").execute(&pool).await?;
        sqlx::query("INSERT INTO receipts VALUES('task','running','thread',NULL)").execute(&pool).await?;
        let writer = pool.clone();
        let update = tokio::spawn(async move {
            sleep(Duration::from_millis(150)).await;
            sqlx::query("UPDATE receipts SET turn_id='turn-exact' WHERE task_id='task'").execute(&writer).await.unwrap();
        });
        let row = bound_receipt(&pool, "task").await?;
        assert_eq!(row.get::<Option<String>,_>("turn_id").as_deref(), Some("turn-exact"), "protocol checkpoint preceded durable binding");
        update.await?;
        Ok(())
    }
}
