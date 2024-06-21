use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

use bytes::Bytes;
use dashmap::DashMap;
use earendil_crypt::{AnonEndpoint, RelayFingerprint, RelayIdentitySecret};
use earendil_packet::{crypt::DhSecret, InnerPacket};
use earendil_topology::RelayGraph;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::{
    config::{InRouteConfig, OutRouteConfig},
    LinkStore,
};

use super::{
    link::Link,
    payment_system::{PaymentSystem, PaymentSystemSelector},
};

pub type ClientId = u64;

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub enum NeighborId {
    Relay(RelayFingerprint),
    Client(ClientId),
}
// TODO: impl Display for NeighborId

#[derive(Clone)]
pub(super) enum LinkNodeId {
    Relay(RelayIdentitySecret),
    Client(ClientId),
}

/// Incoming messages from the link layer that are addressed to "us".
#[derive(Debug)]
pub enum IncomingMsg {
    Forward {
        from: AnonEndpoint,
        body: InnerPacket,
    },
    Backward {
        rb_id: u64,
        body: Bytes,
    },
}

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct LinkPaymentInfo {
    pub price: i64,
    pub debt_limit: i64,
    pub paysystem_name_addrs: Vec<(String, String)>,
}

pub struct LinkConfig {
    pub relay_config: Option<(RelayIdentitySecret, BTreeMap<String, InRouteConfig>)>,
    pub out_routes: BTreeMap<String, OutRouteConfig>,
    pub payment_systems: Vec<Box<dyn PaymentSystem>>,
    pub db_path: PathBuf,
}

#[derive(Clone)]
pub(super) struct LinkNodeCtx {
    pub cfg: Arc<LinkConfig>,
    pub my_id: LinkNodeId,
    pub my_onion_sk: DhSecret,
    pub relay_graph: Arc<RwLock<RelayGraph>>,
    pub link_table: Arc<DashMap<NeighborId, (Arc<Link>, LinkPaymentInfo)>>,
    pub payment_systems: Arc<PaymentSystemSelector>,
    pub store: Arc<LinkStore>,
    pub mel_client: melprot::Client,
}
