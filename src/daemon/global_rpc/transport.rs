use std::time::{Duration, Instant};

use async_trait::async_trait;
use earendil_crypt::Fingerprint;
use futures_util::{future, FutureExt};
use nanorpc::{JrpcRequest, JrpcResponse, RpcTransport};
use smol::Timer;

use crate::daemon::{
    socket::{Endpoint, Socket},
    DaemonContext,
};

use super::GLOBAL_RPC_DOCK;

pub struct GlobalRpcTransport {
    ctx: DaemonContext,
    dest: Fingerprint,
}

impl GlobalRpcTransport {
    pub fn new(ctx: DaemonContext, dest: Fingerprint) -> GlobalRpcTransport {
        GlobalRpcTransport { ctx, dest }
    }
}

#[async_trait]
impl RpcTransport for GlobalRpcTransport {
    type Error = anyhow::Error;

    async fn call_raw(&self, req: JrpcRequest) -> Result<JrpcResponse, Self::Error> {
        let endpoint = Endpoint::new(self.dest, GLOBAL_RPC_DOCK);
        let socket = Socket::bind(self.ctx.clone(), None, None);
        let mut retries = 0;
        let mut timeout: Duration;

        loop {
            socket
                .send_to(serde_json::to_string(&req)?.into(), endpoint)
                .await?;

            timeout = Duration::from_secs(2u64.pow(retries + 1));
            let when = Instant::now() + timeout;
            let timer = Timer::at(when);
            let recv_future = Box::pin(socket.recv_from());

            match future::select(recv_future, timer.fuse()).await {
                future::Either::Left((res, _)) => match res {
                    Ok((res, _endpoint)) => {
                        let jrpc_res: JrpcResponse =
                            serde_json::from_str(&String::from_utf8(res.to_vec())?)?;

                        return Ok(jrpc_res);
                    }
                    Err(_) => {
                        return Err(anyhow::anyhow!("error receiving GlobalRPC response"));
                    }
                },
                future::Either::Right((_, _)) => {
                    retries += 1;
                    continue;
                }
            }
        }
    }
}