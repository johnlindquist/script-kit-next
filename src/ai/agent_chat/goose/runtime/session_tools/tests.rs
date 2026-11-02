use super::super::{ensure_session, Connection, GooseLaunchSpec, Shared};
use serde_json::{json, Value};

/// Feed the real ACP handshake replies over its request channel. Observe when a
/// session actually becomes eligible for warm acquisition or a first prompt.
fn handshake(extensions: Value, load: Value, remaining: &[&str]) -> (bool, Vec<String>) {
    let (out_tx, mut out_rx) = tokio::sync::mpsc::unbounded_channel();
    let mut connection = Connection {
        shared: std::sync::Arc::new(parking_lot::Mutex::new(Shared::default())),
        out_tx,
        next_id: 0,
    };
    let shared = connection.shared.clone();
    let expected_session = "fixture-author";
    let client = async move {
        let mut session = None;
        let mut spec = GooseLaunchSpec::new("/fixture/goose", "/fixture");
        spec.recipe = json!({"extensions":extensions});
        let ready = ensure_session(&mut connection, &spec, &mut session)
            .await
            .is_ok();
        assert_eq!(ready, session.is_some());
        assert_eq!(
            connection.shared.lock().session_id.as_deref(),
            ready.then_some(expected_session)
        );
        drop(connection);
        ready
    };
    let remaining: Vec<String> = remaining.iter().map(|s| (*s).to_owned()).collect();
    let engine = async move {
        let mut methods = Vec::new();
        while let Some(request) = out_rx.recv().await {
            let method = request["method"].as_str().unwrap();
            let params = &request["params"];
            let result = match method {
                "initialize" => json!({"protocolVersion":1}),
                "session/new" => {
                    json!({"sessionId":expected_session,"_meta":{"extensionResults":load}})
                }
                "_goose/unstable/session/extensions/remove" => {
                    assert_eq!(
                        params,
                        &json!({"sessionId":expected_session,"extensionKey":"developer"})
                    );
                    json!({})
                }
                "_goose/unstable/tools/list" => {
                    assert_eq!(
                        params,
                        &json!({"sessionId":expected_session,"extensionName":"developer"})
                    );
                    json!({"tools":remaining.iter().map(|name|json!({"name":name})).collect::<Vec<_>>()})
                }
                "session/set_mode" => json!({}),
                _ => panic!("unexpected call: {method}"),
            };
            methods.push(method.to_owned());
            shared
                .lock()
                .pending
                .remove(&request["id"].as_u64().unwrap())
                .unwrap()
                .send(Ok(result))
                .unwrap();
        }
        methods
    };
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap()
        .block_on(async move {
            let peer = tokio::spawn(engine);
            let ready = client.await;
            (ready, peer.await.unwrap())
        })
}

#[test]
fn restrict_the_actual_session_without_removing_an_explicit_shell_grant() {
    let loaded = json!([{"name":"developer","success":true}]);
    let (ready, calls) = handshake(json!([]), loaded.clone(), &[]);
    assert!(ready);
    assert_eq!(
        calls,
        [
            "initialize",
            "session/new",
            "_goose/unstable/session/extensions/remove",
            "_goose/unstable/tools/list",
            "session/set_mode"
        ]
    );
    let shell = json!([{"name":"developer","available_tools":["shell"]}]);
    let (ready, calls) = handshake(shell.clone(), loaded.clone(), &["shell"]);
    assert!(ready);
    assert!(!calls.iter().any(|x| x.ends_with("/remove")));
    assert!(!handshake(shell, loaded, &["shell", "write"]).0);
}

#[test]
fn no_ready_session_if_a_reported_removal_did_not_remove_the_tools() {
    let (ready, calls) = handshake(
        json!([]),
        json!([{"name":"developer","success":true}]),
        &["write"],
    );
    assert!(!ready);
    assert_eq!(
        calls.last().map(String::as_str),
        Some("_goose/unstable/tools/list")
    );
}

#[test]
fn absent_or_unexpected_inventory_is_not_an_empty_grant() {
    for bad in [
        Value::Null,
        json!([{"name":"ambient","success":true}]),
        json!([{"name":"scriptkit"}]),
        json!([]),
    ] {
        let (ready, calls) = handshake(json!([{"name":"scriptkit"}]), bad, &[]);
        assert!(!ready);
        assert_eq!(calls, ["initialize", "session/new"]);
    }
    assert!(handshake(json!([]), json!([]), &[]).0);
}
