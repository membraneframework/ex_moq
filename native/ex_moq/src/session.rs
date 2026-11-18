use hang::moq_net::origin;

use rustler::{LocalPid, OwnedEnv};
use tokio::task::AbortHandle;
use url::Url;

use crate::{messages, runtime};

pub(crate) struct Handle {
    pub(crate) publish: origin::Producer,
    pub(crate) subscribe: origin::Consumer,
    abort: AbortHandle,
}

impl Handle {
    pub(crate) fn close(&self) {
        self.abort.abort();
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        self.abort.abort();
    }
}

pub(crate) fn create(url: Url, pid: LocalPid, disable_tls_verify: bool) -> Handle {
    let _guard = runtime().handle().enter();
    let publish = moq_tokio::origin::spawn();
    let subscribe = moq_tokio::origin::spawn();

    let publish_consumer = publish.consume();
    let subscribe_consumer = subscribe.consume();

    let task = runtime().spawn(run_session(
        url,
        pid,
        publish_consumer,
        subscribe,
        disable_tls_verify,
    ));

    Handle {
        publish,
        subscribe: subscribe_consumer,
        abort: task.abort_handle(),
    }
}

async fn run_session(
    url: Url,
    pid: LocalPid,
    publish: origin::Consumer,
    subscribe: origin::Producer,
    disable_tls_verify: bool,
) -> Result<(), messages::PidDead> {
    let mut env = OwnedEnv::new();
    match connect(url, publish, subscribe, disable_tls_verify).await {
        Ok(connection) => {
            messages::send_connected(&mut env, pid)?;

            let reason = match connection.closed().await {
                Ok(()) => "closed".to_owned(),
                Err(e) => e.to_string(),
            };
            messages::send_disconnected(&mut env, pid, reason)
        }
        Err(e) => messages::send_setup_failed(&mut env, pid, e.to_string()),
    }
}

async fn connect(
    url: Url,
    publish: origin::Consumer,
    subscribe: origin::Producer,
    disable_tls_verify: bool,
) -> moq_tokio::Result<moq_tokio::Connection> {
    let mut config = moq_tokio::connect::Config::default();
    config.tls.insecure = Some(disable_tls_verify);

    config
        .init(moq_tokio::quic::Config::default())?
        .with_publisher(publish)
        .with_subscriber(subscribe)
        .with_reconnect(false)
        .connect(url)
        .established()
        .await
}
