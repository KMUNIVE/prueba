use crate::event::{UIEvent, WebEvent};
use crate::model::WebModel;
use chrono::offset::TimeZone;
use chrono::{DateTime, Utc};
use futures::future::Either;
use futures::{future, Future, FutureExt, TryFutureExt};
use http::{Method, Request};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::RwLock;
use stremio_analytics::Analytics;
use stremio_core::models::ctx::Ctx;
use stremio_core::models::streaming_server::StreamingServer;
use stremio_core::runtime::msg::{Action, ActionCtx, Event};
use stremio_core::runtime::{Env, EnvError, EnvFuture, EnvFutureExt, TryEnvFuture};
use stremio_core::types::api::AuthRequest;
use stremio_core::types::resource::StreamSource;
use url::Url;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::WorkerGlobalScope;

const UNKNOWN_ERROR: &str = "Unknown Error";
const INSTALLATION_ID_STORAGE_KEY: &str = "installation_id";

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["self"])]
    static app_version: String;
    #[wasm_bindgen(catch, js_namespace = ["self"])]
    static shell_version: Option<String>;
    #[wasm_bindgen(catch, js_namespace = ["self"])]
    async fn get_location_hash() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_namespace = ["self"])]
    async fn local_storage_get_item(key: String) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_namespace = ["self"])]
    async fn local_storage_set_item(key: String, value: String) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_namespace = ["self"])]
    async fn local_storage_remove_item(key: String) -> Result<(), JsValue>;
}

lazy_static! {
    static ref INSTALLATION_ID: RwLock<Option<String>> = Default::default();
    static ref VISIT_ID: String = hex::encode(WebEnv::random_buffer(10));
    static ref ANALYTICS: Analytics<WebEnv> = Default::default();
    static ref PLAYER_REGEX: Regex =
        Regex::new(r"^/player/([^/]*)(?:/([^/]*)/([^/]*)/([^/]*)/([^/]*)/([^/]*))?$").unwrap();
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalyticsContext {
    app_type: String,
    app_version: String,
    server_version: Option<String>,
    shell_version: Option<String>,
    system_language: Option<String>,
    app_language: String,
    #[serde(rename = "installationID")]
    installation_id: String,
    #[serde(rename = "visitID")]
    visit_id: String,
    #[serde(rename = "url")]
    path: String,
}

pub enum WebEnv {}

impl WebEnv {
    pub fn init() -> TryEnvFuture<()> {
        WebEnv::migrate_storage_schema()
            .and_then(|_| WebEnv::get_storage::<String>(INSTALLATION_ID_STORAGE_KEY))
            .map_ok(|installation_id| {
                installation_id.or_else(|| Some(hex::encode(WebEnv::random_buffer(10))))
            })
            .and_then(|installation_id| {
                *INSTALLATION_ID
                    .write()
                    .expect("installation id write failed") = installation_id;
                WebEnv::set_storage(
                    INSTALLATION_ID_STORAGE_KEY,
                    Some(&*INSTALLATION_ID.read().expect("installation id read failed")),
                )
            })
            .inspect_ok(|_| {
                WebEnv::set_interval(
                    || WebEnv::exec_concurrent(WebEnv::send_next_analytics_batch()),
                    30 * 1000,
                );
            })
            .boxed_local()
    }
    pub fn get_location_hash() -> EnvFuture<'static, String> {
        get_location_hash()
            .map(|location_hash| {
                location_hash
                    .ok()
                    .and_then(|location_hash| location_hash.as_string())
                    .unwrap_or_default()
            })
            .boxed_env()
    }
    pub fn emit_to_analytics(event: &WebEvent, model: &WebModel, path: &str) {
        let (name, data) = match event {
            WebEvent::UIEvent(UIEvent::LocationPathChanged { prev_path }) => (
                "stateChange".to_owned(),
                json!({ "previousURL": sanitize_location_path(prev_path) }),
            ),
            WebEvent::UIEvent(UIEvent::Search {
                query,
                responses_count,
            }) => (
                "search".to_owned(),
                json!({ "query": query, "rows": responses_count }),
            ),
            WebEvent::UIEvent(UIEvent::Share { url }) => {
                ("share".to_owned(), json!({ "url": url }))
            }
            WebEvent::UIEvent(UIEvent::StreamClicked { stream }) => (
                "streamClicked".to_owned(),
                json!({
                    "type": match &stream.source {
                        StreamSource::Url { .. } => "Url",
                        StreamSource::YouTube { .. } => "YouTube",
                        StreamSource::Torrent { .. } => "Torrent",
                        StreamSource::External { .. } => "External",
                        StreamSource::PlayerFrame { .. } => "PlayerFrame"
                    }
                }),
            ),
            WebEvent::CoreEvent(core_event) => match core_event.as_ref() {
                Event::UserAuthenticated { auth_request } => (
                    "login".to_owned(),
                    json!({
                        "type": match auth_request {
                            AuthRequest::Login { facebook, .. } if *facebook => "facebook",
                            AuthRequest::Login { .. } => "login",
                            AuthRequest::LoginWithToken { .. } => "loginWithToken",
                            AuthRequest::Register { .. } => "register",
                        },
                    }),
                ),
                Event::AddonInstalled { transport_url, id } => (
                    "installAddon".to_owned(),
                    json!({
                        "addonTransportUrl": transport_url,
                        "addonID": id
                    }),
                ),
                Event::AddonUninstalled { transport_url, id } => (
                    "removeAddon".to_owned(),
                    json!({
                        "addonTransportUrl": transport_url,
                        "addonID": id
                    }),
                ),
                Event::PlayerPlaying { load_time, context } => (
                    "playerPlaying".to_owned(),
                    json!({
                        "loadTime": load_time,
                        "player": context
                    }),
                ),
                Event::PlayerStopped { context } => {
                    ("playerStopped".to_owned(), json!({ "player": context }))
                }
                Event::PlayerEnded {
                    context,
                    is_binge_enabled,
                    is_playing_next_video,
                } => (
                    "playerEnded".to_owned(),
                    json!({
                       "player": context,
                       "isBingeEnabled": is_binge_enabled,
                       "isPlayingNextVideo": is_playing_next_video
                    }),
                ),
                Event::TraktPlaying { context } => {
                    ("traktPlaying".to_owned(), json!({ "player": context }))
                }
                Event::TraktPaused { context } => {
                    ("traktPaused".to_owned(), json!({ "player": context }))
                }
                _ => return,
            },
            WebEvent::CoreAction(core_action) => match core_action.as_ref() {
                Action::Ctx(ActionCtx::AddToLibrary(meta_preview)) => {
                    let library_item = model.ctx.library.items.get(&meta_preview.id);
                    (
                        "addToLib".to_owned(),
                        json!({
                            "libItemID":  &meta_preview.id,
                            "libItemType": &meta_preview.r#type,
                            "libItemName": &meta_preview.name,
                            "wasTemp": library_item.map(|library_item| library_item.temp).unwrap_or_default(),
                            "isReadded": library_item.map(|library_item| library_item.removed).unwrap_or_default(),
                        }),
                    )
                }
                Action::Ctx(ActionCtx::RemoveFromLibrary(id)) => {
                    match model.ctx.library.items.get(id) {
                        Some(library_item) => (
                            "removeFromLib".to_owned(),
                            json!({
                                "libItemID":  &library_item.id,
                                "libItemType": &library_item.r#type,
                                "libItemName": &library_item.name,
                            }),
                        ),
                        _ => return,
                    }
                }
                Action::Ctx(ActionCtx::Logout) => ("logout".to_owned(), serde_json::Value::Null),
                _ => return,
            },
        };
        ANALYTICS.emit(name, data, &model.ctx, &model.streaming_server, path);
    }
    pub fn send_next_analytics_batch() -> impl Future<Output = ()> {
        ANALYTICS.send_next_batch()
    }
    pub fn set_interval<F: FnMut() + 'static>(func: F, timeout: i32) -> i32 {
        let func = Closure::wrap(Box::new(func) as Box<dyn FnMut()>);
        let interval_id = global()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                func.as_ref().unchecked_ref(),
                timeout,
            )
            .expect("set interval failed");
        func.forget();
        interval_id
    }
    #[allow(dead_code)]
    pub fn clear_interval(id: i32) {
        global().clear_interval_with_handle(id);
    }
    pub fn random_buffer(len: usize) -> Vec<u8> {
        let mut buffer = vec![0u8; len];
        getrandom::getrandom(buffer.as_mut_slice()).expect("generate random buffer failed");
        buffer
    }
}

impl Env for WebEnv {
    fn fetch<IN, OUT>(request: Request<IN>) -> TryEnvFuture<OUT>
    where
        IN: Serialize,
        for<'de> OUT: Deserialize<'de> + 'static,
    {
        let (parts, body) = request.into_parts();
        let url = parts.uri.to_string();
        let method = parts.method.as_str();
        let headers = {
            let mut headers = HashMap::new();
            for (key, value) in parts.headers.iter() {
                let key = key.as_str().to_owned();
                let value = String::from_utf8_lossy(value.as_bytes()).into_owned();
                headers.entry(key).or_insert_with(Vec::new).push(value);
            }
            JsValue::from_serde(&headers).unwrap()
        };
        let body = match serde_json::to_string(&body) {
            Ok(ref body) if body != "null" && parts.method != Method::GET => {
                Some(JsValue::from_str(body))
            }
            _ => None,
        };
        let mut request_options = web_sys::RequestInit::new();
        request_options
            .method(method)
            .headers(&headers)
            .body(body.as_ref());
        let request = web_sys::Request::new_with_str_and_init(&url, &request_options)
            .expect("request builder failed");
        let promise = global().fetch_with_request(&request);
        JsFuture::from(promise)
            .map_err(|error| {
                EnvError::Fetch(
                    error
                        .dyn_into::<js_sys::Error>()
                        .map(|error| String::from(error.message()))
                        .unwrap_or_else(|_| UNKNOWN_ERROR.to_owned()),
                )
            })
            .and_then(|resp| {
                let resp = resp.dyn_into::<web_sys::Response>().unwrap();
                if resp.status() != 200 {
                    Either::Right(future::err(EnvError::Fetch(format!(
                        "Unexpected HTTP status code {}",
                        resp.status(),
                    ))))
                } else {
                    Either::Left(JsFuture::from(resp.json().unwrap()).map_err(|error| {
                        EnvError::Fetch(
                            error
                                .dyn_into::<js_sys::Error>()
                                .map(|error| String::from(error.message()))
                                .unwrap_or_else(|_| UNKNOWN_ERROR.to_owned()),
                        )
                    }))
                }
            })
            .and_then(|resp| {
                cfg_if::cfg_if! {
                    if #[cfg(debug_assertions)] {
                        future::ready(
                            js_sys::JSON::stringify(&resp)
                                .map_err(|error| {
                                    EnvError::Fetch(
                                        error
                                            .dyn_into::<js_sys::Error>()
                                            .map(|error| String::from(error.message()))
                                            .unwrap_or_else(|_| UNKNOWN_ERROR.to_owned()),
                                    )
                                })
                                .and_then(|resp| {
                                    let resp = Into::<String>::into(resp);
                                    let mut deserializer =
                                        serde_json::Deserializer::from_str(resp.as_str());
                                    serde_path_to_error::deserialize::<_, OUT>(&mut deserializer)
                                        .map_err(|error| EnvError::Fetch(error.to_string()))
                                }),
                        )
                    } else {
                        future::ready(resp.into_serde().map_err(EnvError::from))
                    }
                }
            })
            .boxed_local()
    }
    fn get_storage<T>(key: &str) -> TryEnvFuture<Option<T>>
    where
        for<'de> T: Deserialize<'de> + 'static,
    {
        local_storage_get_item(key.to_owned())
            .map_err(|error| {
                EnvError::StorageReadError(
                    error
                        .dyn_into::<js_sys::Error>()
                        .map(|error| String::from(error.message()))
                        .unwrap_or_else(|_| UNKNOWN_ERROR.to_owned()),
                )
            })
            .and_then(|value| async move {
                value
                    .as_string()
                    .map(|value| serde_json::from_str(&value))
                    .transpose()
                    .map_err(EnvError::from)
            })
            .boxed_local()
    }
    fn set_storage<T: Serialize>(key: &str, value: Option<&T>) -> TryEnvFuture<()> {
        let key = key.to_owned();
        match value {
            Some(value) => future::ready(serde_json::to_string(value))
                .map_err(EnvError::from)
                .and_then(|value| {
                    local_storage_set_item(key, value).map_err(|error| {
                        EnvError::StorageWriteError(
                            error
                                .dyn_into::<js_sys::Error>()
                                .map(|error| String::from(error.message()))
                                .unwrap_or_else(|_| UNKNOWN_ERROR.to_owned()),
                        )
                    })
                })
                .boxed_local(),
            None => local_storage_remove_item(key)
                .map_err(|error| {
                    EnvError::StorageWriteError(
                        error
                            .dyn_into::<js_sys::Error>()
                            .map(|error| String::from(error.message()))
                            .unwrap_or_else(|_| UNKNOWN_ERROR.to_owned()),
                    )
                })
                .boxed_local(),
        }
    }
    fn exec_concurrent<F>(future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        spawn_local(future)
    }
    fn exec_sequential<F>(future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        spawn_local(future)
    }
    fn now() -> DateTime<Utc> {
        let msecs = js_sys::Date::now() as i64;
        let (secs, nsecs) = (msecs / 1000, msecs % 1000 * 1_000_000);
        Utc.timestamp_opt(secs, nsecs as u32)
            .single()
            .expect("Invalid timestamp")
    }
    fn flush_analytics() -> EnvFuture<'static, ()> {
        ANALYTICS.flush().boxed_local()
    }
    fn analytics_context(
        ctx: &Ctx,
        streaming_server: &StreamingServer,
        path: &str,
    ) -> serde_json::Value {
        serde_json::to_value(AnalyticsContext {
            app_type: "stremio-web".to_owned(),
            app_version: app_version.to_owned(),
            server_version: streaming_server
                .settings
                .as_ref()
                .ready()
                .map(|settings| settings.server_version.to_owned()),
            shell_version: shell_version.to_owned(),
            system_language: global()
                .navigator()
                .language()
                .map(|language| language.to_lowercase()),
            app_language: ctx.profile.settings.interface_language.to_owned(),
            installation_id: INSTALLATION_ID
                .read()
                .expect("installation id read failed")
                .as_ref()
                .expect("installation id not available")
                .to_owned(),
            visit_id: VISIT_ID.to_owned(),
            path: sanitize_location_path(path),
        })
        .unwrap()
    }
    #[cfg(debug_assertions)]
    fn log(message: String) {
        web_sys::console::log_1(&JsValue::from(message));
    }
}

fn sanitize_location_path(path: &str) -> String {
    match Url::parse(&format!("stremio://{}", path)) {
        Ok(url) => {
            let query = url
                .query()
                .map(|query| format!("?{}", query))
                .unwrap_or_default();
            let path = match PLAYER_REGEX.captures(url.path()) {
                Some(captures) => {
                    if captures.get(3).is_some()
                        && captures.get(4).is_some()
                        && captures.get(5).is_some()
                        && captures.get(6).is_some()
                    {
                        format!(
                            "/player/***/***/{}/{}/{}/{}",
                            captures.get(3).unwrap().as_str(),
                            captures.get(4).unwrap().as_str(),
                            captures.get(5).unwrap().as_str(),
                            captures.get(6).unwrap().as_str(),
                        )
                    } else {
                        "/player/***".to_owned()
                    }
                }
                _ => url.path().to_owned(),
            };
            format!("{}{}", path, query)
        }
        _ => path.to_owned(),
    }
}

fn global() -> WorkerGlobalScope {
    js_sys::global()
        .dyn_into::<WorkerGlobalScope>()
        .expect("worker global scope is not available")
}
