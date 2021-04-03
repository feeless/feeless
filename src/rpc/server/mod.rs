use tokio::sync::mpsc;
use tokio::sync::oneshot;
use warp::Filter;

pub struct RPCCommand {
    response_rx: (),
    state: ArcState,
}

pub struct RPCServer {
    tx: mpsc::Sender<RPCCommand>,
}

impl RPCServer {
    pub fn new_with_rx(state: ArcState) -> (Self, mpsc::Receiver<RPCCommand>) {
        let (tx, rx) = mpsc::channel::<RPCCommand>(100);
        let s = Self { tx, state };
        (s, rx)
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let rpc = warp::post()
            .and(warp::body::content_length_limit(1024 * 16))
            .and(warp::body::json())
            .map(|name| format!("Hello, {}!", name));
        warp::serve(rpc).run(([127, 0, 0, 1], 7076)).await;
        Ok(())
    }
}
