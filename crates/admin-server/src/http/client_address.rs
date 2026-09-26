//! The client's address: the peer's, unless the peer is a trusted proxy.

use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::extract::connect_info::MockConnectInfo;
use axum::extract::{ConnectInfo, FromRequestParts};
use axum::http::request::Parts;
use axum::http::{Extensions, HeaderMap};

use crate::config::TrustedProxies;
use crate::state::AppState;

/// The header a proxy appends the address it received a request from to.
const FORWARDED_FOR: &str = "x-forwarded-for";

/// The address a request comes from. With a peer in `LPA_TRUSTED_PROXIES`, it is the right-most
/// address of `X-Forwarded-For` that is not a trusted proxy; otherwise, the peer's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ClientAddress(pub IpAddr);

impl ClientAddress {
    /// The client address of a request from `peer` carrying `headers`.
    #[must_use]
    pub fn resolve(peer: IpAddr, headers: &HeaderMap, proxies: &TrustedProxies) -> Self {
        if !proxies.contains(peer) {
            return Self(peer);
        }
        let forwarded = forwarded_addresses(headers);
        let client = forwarded
            .iter()
            .rev()
            .map_while(|hop| hop.parse::<IpAddr>().ok())
            .find(|address| !proxies.contains(*address));
        Self(client.unwrap_or(peer))
    }
}

impl FromRequestParts<AppState> for ClientAddress {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Infallible> {
        let peer = peer_address(&parts.extensions);
        Ok(Self::resolve(
            peer,
            &parts.headers,
            &state.config.trusted_proxies,
        ))
    }
}

/// The address of the connection's peer, as `ConnectInfo` finds it — a test's
/// `MockConnectInfo` included —; unspecified without connection information.
pub(crate) fn peer_address(extensions: &Extensions) -> IpAddr {
    let connected = extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0);
    let mocked = || {
        extensions
            .get::<MockConnectInfo<SocketAddr>>()
            .map(|info| info.0)
    };
    connected
        .or_else(mocked)
        .map_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED), |peer| peer.ip())
}

/// The hops of every `X-Forwarded-For` header, left to right.
fn forwarded_addresses(headers: &HeaderMap) -> Vec<String> {
    headers
        .get_all(FORWARDED_FOR)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .map(|hop| hop.trim().to_owned())
        .collect()
}
