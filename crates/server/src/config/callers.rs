//! Who may call: the oldest client version of each platform, and the proxies whose
//! `X-Forwarded-For` is believed.

use std::collections::BTreeMap;
use std::net::IpAddr;

use ipnet::IpNet;
use semver::Version;

use super::env::FromVariable;

/// `LP_MIN_CLIENT_VERSIONS`: the oldest version each platform may use, as
/// `web=1.0.0,desktop=1.0.0`. A platform it does not list has no minimum.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ClientVersions(BTreeMap<String, Version>);

impl ClientVersions {
    /// The oldest version `platform` may use, if it has one.
    #[must_use]
    pub fn minimum(&self, platform: &str) -> Option<&Version> {
        self.0.get(platform)
    }
}

impl FromVariable for ClientVersions {
    const EXPECTED: &'static str = "platform=version pairs such as web=1.0.0,desktop=1.0.0";

    fn from_variable(value: &str) -> Option<Self> {
        value
            .split(',')
            .map(|pair| {
                let (platform, version) = pair.trim().split_once('=')?;
                let platform = platform.trim();
                let version = Version::parse(version.trim()).ok()?;
                (!platform.is_empty()).then(|| (platform.to_owned(), version))
            })
            .collect::<Option<_>>()
            .map(Self)
    }
}

/// `LP_TRUSTED_PROXIES`: the networks of the proxies in front of the server.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TrustedProxies(Vec<IpNet>);

impl TrustedProxies {
    /// Whether `address` belongs to a trusted proxy.
    #[must_use]
    pub fn contains(&self, address: IpAddr) -> bool {
        self.0.iter().any(|network| network.contains(&address))
    }
}

impl FromVariable for TrustedProxies {
    const EXPECTED: &'static str = "networks separated by commas, such as 172.16.0.0/12,10.0.0.1";

    fn from_variable(value: &str) -> Option<Self> {
        value
            .split(',')
            .map(|network| parse_network(network.trim()))
            .collect::<Option<_>>()
            .map(Self)
    }
}

/// A network in CIDR notation, or a single address.
fn parse_network(value: &str) -> Option<IpNet> {
    value
        .parse()
        .ok()
        .or_else(|| value.parse::<IpAddr>().ok().map(IpNet::from))
}
