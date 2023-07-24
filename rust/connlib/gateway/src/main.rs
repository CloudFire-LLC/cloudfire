use anyhow::{Context, Result};
use std::{net::Ipv4Addr, str::FromStr};

use firezone_gateway_connlib::{Callbacks, Error, ResourceDescription, Session, TunnelAddresses};
use url::Url;

#[derive(Clone)]
pub struct CallbackHandler;

impl Callbacks for CallbackHandler {
    fn on_set_interface_config(&self, _tunnel_addresses: TunnelAddresses, _dns_address: Ipv4Addr) {}

    fn on_tunnel_ready(&self) {
        tracing::trace!("Tunnel connected with address");
    }

    fn on_add_route(&self, _route: String) {}

    fn on_remove_route(&self, _route: String) {}

    fn on_update_resources(&self, resource_list: Vec<ResourceDescription>) {
        tracing::trace!("Resources updated, current list: {resource_list:?}");
    }

    fn on_disconnect(&self, error: Option<&Error>) {
        tracing::trace!("Tunnel disconnected: {error:?}");
    }

    fn on_error(&self, error: &Error) {
        tracing::warn!("Encountered recoverable error: {error}");
    }
}

const URL_ENV_VAR: &str = "FZ_URL";
const SECRET_ENV_VAR: &str = "FZ_SECRET";

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    // TODO: allow passing as arg vars
    let url = parse_env_var::<Url>(URL_ENV_VAR)?;
    let secret = parse_env_var::<String>(SECRET_ENV_VAR)?;
    let mut session = Session::connect(url, secret, CallbackHandler).unwrap();
    session.wait_for_ctrl_c().unwrap();
    session.disconnect(None);
    Ok(())
}

fn parse_env_var<T>(key: &str) -> Result<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    let res = std::env::var(key)
        .with_context(|| format!("`{key}` env variable is unset"))?
        .parse()
        .with_context(|| format!("failed to parse {key} env variable"))?;

    Ok(res)
}
