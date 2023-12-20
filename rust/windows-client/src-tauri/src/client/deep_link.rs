//! A module for registering, catching, and parsing deep links that are sent over to the app's already-running instance
//! Based on reading some of the Windows code from <https://github.com/FabianLars/tauri-plugin-deep-link>, which is licensed "MIT OR Apache-2.0"

use secrecy::SecretString;
use std::{ffi::c_void, io, path::Path};
use tokio::{io::AsyncReadExt, io::AsyncWriteExt, net::windows::named_pipe};
use windows::Win32::Security as WinSec;

pub(crate) const FZ_SCHEME: &str = "firezone-fd0020211111";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Error from client's POV
    #[error(transparent)]
    ClientCommunications(io::Error),
    /// Error while connecting to the server
    #[error(transparent)]
    Connect(io::Error),
    /// Something went wrong finding the path to our own exe
    #[error(transparent)]
    CurrentExe(io::Error),
    /// We got some data but it's not UTF-8
    #[error(transparent)]
    LinkNotUtf8(std::string::FromUtf8Error),
    #[error("named pipe server couldn't start listening, we are probably the second instance")]
    Listen,
    /// Error from server's POV
    #[error(transparent)]
    ServerCommunications(io::Error),
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
    /// Something went wrong setting up the registry
    #[error(transparent)]
    WindowsRegistry(io::Error),
}

pub(crate) struct AuthCallback {
    pub actor_name: String,
    pub token: SecretString,
    pub _identifier: SecretString,
}

pub(crate) fn parse_auth_callback(url: &url::Url) -> Option<AuthCallback> {
    match url.host() {
        Some(url::Host::Domain("handle_client_auth_callback")) => {}
        _ => return None,
    }
    if url.path() != "/" {
        return None;
    }

    let mut actor_name = None;
    let mut token = None;
    let mut identifier = None;

    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "actor_name" => {
                if actor_name.is_some() {
                    // actor_name must appear exactly once
                    return None;
                }
                actor_name = Some(value.to_string());
            }
            "client_auth_token" => {
                if token.is_some() {
                    // client_auth_token must appear exactly once
                    return None;
                }
                token = Some(SecretString::new(value.to_string()));
            }
            "identity_provider_identifier" => {
                if identifier.is_some() {
                    // identity_provider_identifier must appear exactly once
                    return None;
                }
                identifier = Some(SecretString::new(value.to_string()));
            }
            _ => {}
        }
    }

    Some(AuthCallback {
        actor_name: actor_name?,
        token: token?,
        _identifier: identifier?,
    })
}

/// A server for a named pipe, so we can receive deep links from other instances
/// of the client launched by web browsers
pub struct Server {
    inner: named_pipe::NamedPipeServer,
}

impl Server {
    /// Construct a server, but don't await client connections yet
    ///
    /// Panics if there is no Tokio runtime
    pub fn new(id: &str) -> Result<Self, Error> {
        // This isn't air-tight - We recreate the whole server on each loop,
        // rather than binding 1 socket and accepting many streams like a normal socket API.
        // I can only assume Tokio is following Windows' underlying API.

        // We could instead pick an ephemeral TCP port and write that to a file,
        // akin to how Unix processes will write their PID to a file to manage long-running instances
        // But this doesn't require us to listen on TCP.

        let mut server_options = named_pipe::ServerOptions::new();
        server_options.first_pipe_instance(true);

        // This will allow non-admin clients to connect to us even if we're running as admin
        let mut sd = WinSec::SECURITY_DESCRIPTOR::default();
        let psd = WinSec::PSECURITY_DESCRIPTOR(&mut sd as *mut _ as *mut c_void);
        unsafe {
            // ChatGPT pointed me to these functions, it's better than the official MS docs
            WinSec::InitializeSecurityDescriptor(
                psd,
                windows::Win32::System::SystemServices::SECURITY_DESCRIPTOR_REVISION,
            )
            .map_err(|_| Error::Listen)?;
            WinSec::SetSecurityDescriptorDacl(psd, true, None, false).map_err(|_| Error::Listen)?;
        }

        let mut sa = WinSec::SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<WinSec::SECURITY_ATTRIBUTES>()
                .try_into()
                .unwrap(),
            lpSecurityDescriptor: psd.0,
            bInheritHandle: false.into(),
        };

        let path = named_pipe_path(id);
        let server = unsafe {
            server_options
                .create_with_security_attributes_raw(path, &mut sa as *mut _ as *mut c_void)
        }
        .map_err(|_| Error::Listen)?;

        tracing::debug!("server is bound");
        Ok(Server { inner: server })
    }

    /// Await one incoming deep link from a named pipe client
    /// Tokio's API is strange, so this consumes the server.
    /// I assume this is based on the underlying Windows API.
    /// I tried re-using the server and it acted strange. The official Tokio
    /// examples are not clear on this.
    pub async fn accept(mut self) -> Result<url::Url, Error> {
        self.inner
            .connect()
            .await
            .map_err(Error::ServerCommunications)?;
        tracing::debug!("server got connection");

        // TODO: Limit the read size here. Our typical callback is 350 bytes, so 4,096 bytes should be more than enough.
        // Also, I think `read_to_end` can do partial reads because this is a network socket,
        // not a file. We might need a length-prefixed or newline-terminated format for IPC.
        let mut bytes = vec![];
        self.inner
            .read_to_end(&mut bytes)
            .await
            .map_err(Error::ServerCommunications)?;

        self.inner.disconnect().ok();

        tracing::debug!("Server read");
        let s = String::from_utf8(bytes).map_err(Error::LinkNotUtf8)?;
        tracing::info!("{}", s);
        let url = url::Url::parse(&s)?;

        Ok(url)
    }
}

/// Open a deep link by sending it to the already-running instance of the app
pub async fn open(id: &str, url: &url::Url) -> Result<(), Error> {
    let path = named_pipe_path(id);
    let mut client = named_pipe::ClientOptions::new()
        .open(path)
        .map_err(Error::Connect)?;
    client
        .write_all(url.as_str().as_bytes())
        .await
        .map_err(Error::ClientCommunications)?;
    Ok(())
}

/// Registers the current exe as the handler for our deep link scheme.
///
/// This is copied almost verbatim from tauri-plugin-deep-link's `register` fn, with an improvement
/// that we send the deep link to a subcommand so the URL won't confuse `clap`
///
/// * `id` A unique ID for the app, e.g. "com.contoso.todo-list" or "dev.firezone.client"
pub fn register(id: &str) -> Result<(), Error> {
    let exe = tauri_utils::platform::current_exe()
        .map_err(Error::CurrentExe)?
        .display()
        .to_string()
        .replace("\\\\?\\", "");

    set_registry_values(id, &exe).map_err(Error::WindowsRegistry)?;

    Ok(())
}

/// Set up the Windows registry to call the given exe when our deep link scheme is used
///
/// All errors from this function are registry-related
fn set_registry_values(id: &str, exe: &str) -> Result<(), io::Error> {
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let base = Path::new("Software").join("Classes").join(FZ_SCHEME);

    let (key, _) = hkcu.create_subkey(&base)?;
    key.set_value("", &format!("URL:{}", id))?;
    key.set_value("URL Protocol", &"")?;

    let (icon, _) = hkcu.create_subkey(base.join("DefaultIcon"))?;
    icon.set_value("", &format!("{},0", &exe))?;

    let (cmd, _) = hkcu.create_subkey(base.join("shell").join("open").join("command"))?;
    cmd.set_value("", &format!("{} open-deep-link \"%1\"", &exe))?;

    Ok(())
}

fn named_pipe_path(id: &str) -> String {
    format!(r"\\.\pipe\{}", id)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use secrecy::ExposeSecret;

    #[test]
    fn parse_auth_callback() -> Result<()> {
        let input = "firezone://handle_client_auth_callback/?actor_name=Reactor+Scram&client_auth_token=a_very_secret_string&identity_provider_identifier=12345";
        let input = url::Url::parse(input)?;
        dbg!(&input);
        let actual = super::parse_auth_callback(&input).unwrap();

        assert_eq!(actual.actor_name, "Reactor Scram");
        assert_eq!(actual.token.expose_secret(), "a_very_secret_string");

        let input = "firezone://not_handle_client_auth_callback/?actor_name=Reactor+Scram&client_auth_token=a_very_secret_string&identity_provider_identifier=12345";
        let actual = super::parse_auth_callback(&url::Url::parse(input)?);
        assert!(actual.is_none());

        Ok(())
    }
}
