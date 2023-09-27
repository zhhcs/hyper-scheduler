use super::{
    CallConfigRequest, CallFuncResponse, CallWithName, RegisterResponse, StatusQuery, TestRequest,
};
use crate::{
    axum::get_port,
    result::{FuncResult, ResultFuture},
    runtime::Runtime,
    runwasm::{
        call_func, call_func_sync, get_status_by_name, get_test_env, set_test_env, Environment,
        FuncConfig, RegisterConfig, Tester,
    },
};
use axum::{
    extract::{Multipart, Query},
    routing::{get, post},
    Json, Router,
};
use std::{cell::RefCell, collections::HashMap, net::SocketAddr, sync::Arc};

thread_local! {
    static ENV_MAP: RefCell<HashMap<String, Environment>> = RefCell::new(HashMap::new());
}

lazy_static::lazy_static! {
    static ref RUNTIME: Arc<Runtime> = Arc::new(Runtime::new());
}

pub struct Server {}

impl Server {
    pub async fn start() {
        RUNTIME.as_ref();
        let app = Router::new()
            .route("/register", post(Self::register))
            .route("/test", post(Self::test))
            .route("/call_with_name", post(Self::call_with_name))
            .route("/init", get(Self::init))
            .route("/call", post(Self::call_func))
            .route("/status", get(Self::get_status))
            .route("/uname", get(Self::get_status_by_name));

        let addr = SocketAddr::from(([0, 0, 0, 0], get_port()));
        tracing::info!("listening on {}", addr);

        let handle = tokio::task::spawn_blocking(|| loop {
            std::thread::sleep(std::time::Duration::from_millis(10));
            if let Some(tester) = get_test_env() {
                // tracing::info!("call_func_sync tid {}", gettid());
                match call_func_sync(tester.env) {
                    Ok(time) => {
                        let res = format!("{:?}", time.as_millis() + 1);
                        tester.result.set_result(&res);
                        tester.result.set_completed();
                    }
                    Err(err) => {
                        tester.result.set_result(&err.to_string());
                        tester.result.set_completed();
                    }
                };
            }
        });

        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
        handle.await.unwrap();
    }

    /// route: /register
    async fn register(mut multipart: Multipart) -> Json<RegisterResponse> {
        // tracing::info!("register tid = {}", nix::unistd::gettid());
        let mut reponse = RegisterResponse {
            status: "Error".to_owned(),
            url: "null".to_owned(),
        };
        if let Some(field) = multipart.next_field().await.unwrap() {
            let name = field.name().unwrap().to_string();
            if ENV_MAP.with(|map| map.borrow().contains_key(&name)) {
                reponse.status.push_str("_Invalid_wasm_name");
                return Json(reponse);
            }
            let data = field.bytes().await.unwrap();
            let path = format!("/tmp/{}", name);
            tokio::fs::write(&path, &data).await.unwrap();
            tracing::info!("saved to: {}", path);

            let config = RegisterConfig::new(&path, &name);
            if let Ok(env) = Environment::new(&config) {
                ENV_MAP.with(|map| {
                    map.borrow_mut()
                        .insert((&env.get_wasm_name()).to_string(), env)
                });

                reponse.status = "Success".to_owned();
                reponse.url = format!("http://127.0.0.1:{}/call", get_port());
                return Json(reponse);
            };
        }
        Json(reponse)
    }

    async fn test(Json(test_config): Json<TestRequest>) -> Json<CallFuncResponse> {
        // tracing::info!("test tid = {}", nix::unistd::gettid());
        let mut response = CallFuncResponse {
            status: "Error".to_owned(),
            result: "null".to_owned(),
        };
        let name = test_config.wasm_name.to_owned();
        let mut status = false;

        match FuncConfig::from(test_config) {
            Ok(func_config) => {
                let func_result = Arc::new(FuncResult::new());
                ENV_MAP.with(|map| {
                    if let Some(env) = map.borrow_mut().get_mut(&name) {
                        env.set_func_config(func_config);
                        let env = env.clone();
                        set_test_env(Tester {
                            env,
                            result: func_result.clone(),
                        });
                        status = true;
                    } else {
                        response.status = "Error_Invalid_wasm_name".to_owned();
                    }
                });
                if status {
                    let res = Self::get_result(&func_result).await;
                    if let Ok(time) = res.parse::<u64>() {
                        ENV_MAP.with(|map| {
                            map.borrow_mut().get_mut(&name).unwrap().set_test_time(time)
                        });
                        response.result = time.to_string();
                    } else {
                        response.result = res;
                    }
                    response.status = "Success".to_owned();
                }
            }
            Err(err) => response.status = format!("Error_{}", err),
        };

        Json(response)
    }

    /// route: /init
    async fn init(Json(config): Json<RegisterConfig>) -> Json<RegisterResponse> {
        let mut reponse = RegisterResponse {
            status: "Error".to_owned(),
            url: "null".to_owned(),
        };

        let name = config.get_wasm_name();
        if ENV_MAP.with(|map| map.borrow().contains_key(name)) {
            reponse.status.push_str("_Invalid_wasm_name");
            return Json(reponse);
        }

        let path = config.get_path();
        let config = RegisterConfig::new(&path, &name);
        if let Ok(env) = Environment::new(&config) {
            ENV_MAP.with(|map| {
                map.borrow_mut()
                    .insert((&env.get_wasm_name()).to_string(), env)
            });

            reponse.status = "Success".to_owned();
            reponse.url = format!("http://127.0.0.1:{}/call", get_port());
            return Json(reponse);
        }
        Json(reponse)
    }

    async fn call_with_name(Json(name): Json<CallWithName>) -> Json<CallFuncResponse> {
        let mut response = CallFuncResponse {
            status: "Error".to_owned(),
            result: "null".to_owned(),
        };
        let mut status = false;
        let func_result = Arc::new(FuncResult::new());
        ENV_MAP.with(|map| {
            if let Some(env) = map.borrow().get(&name.wasm_name) {
                if let Some(func_config) = env.get_func_config() {
                    let env = env.clone();
                    match call_func(&RUNTIME, env, func_config, &func_result) {
                        Ok(_) => {
                            status = true;
                        }
                        Err(err) => response.status = format!("Error_{}", err),
                    };
                }
            } else {
                response.status = "Error_Invalid_wasm_name".to_owned();
            }
        });
        if status {
            let res = Self::get_result(&func_result).await;
            response.status = "Success".to_owned();
            response.result = res;
        }
        Json(response)
    }

    /// route: /call
    async fn call_func(Json(call_config): Json<CallConfigRequest>) -> Json<CallFuncResponse> {
        // tracing::info!("call tid = {}", nix::unistd::gettid());
        let mut response = CallFuncResponse {
            status: "Error".to_owned(),
            result: "null".to_owned(),
        };
        let name = call_config.wasm_name.to_owned();
        let mut status = false;

        match FuncConfig::new(call_config) {
            Ok(func_config) => {
                let func_result = Arc::new(FuncResult::new());
                ENV_MAP.with(|map| {
                    if let Some(env) = map.borrow().get(&name) {
                        tracing::info!("{:?}", env.get_func_config());
                        let test_time = env.get_test_time();
                        if func_config.get_relative_deadline() >= test_time {
                            let env = env.clone();
                            match call_func(&RUNTIME, env, func_config, &func_result) {
                                Ok(_) => {
                                    status = true;
                                }
                                Err(err) => response.status = format!("Error_{}", err),
                            };
                        }
                    } else {
                        response.status = "Error_Invalid_wasm_name".to_owned();
                    }
                });
                if status {
                    let res = Self::get_result(&func_result).await;
                    response.status = "Success".to_owned();
                    response.result = res;
                }
            }
            Err(err) => response.status = format!("Error_{}", err),
        };

        Json(response)
    }

    async fn get_result(func_result: &Arc<FuncResult>) -> String {
        // tracing::info!("get_result tid = {}", nix::unistd::gettid());
        ResultFuture {
            result: func_result.clone(),
        }
        .await
    }

    /// route: /status
    async fn get_status() -> String {
        if let Some(status) = RUNTIME.get_status() {
            if status.len() == 0 {
                return "no task running".to_string();
            }
            let mut s = String::new();
            status.iter().for_each(|(id, stat)| {
                s.push_str(&format!(
                    "\nid: {}, status: {:?}\n{}",
                    id, stat.co_status, stat
                ));
            });
            return s;
        };
        "500".to_string()
    }

    /// route: /uname
    async fn get_status_by_name(Query(params): Query<StatusQuery>) -> String {
        if let Some(status) = get_status_by_name(&RUNTIME, &params.uname) {
            format!(
                "\nuname: {} {:?}\n{}",
                params.uname, status.co_status, status
            )
        } else {
            "not found".to_string()
        }
    }
}
