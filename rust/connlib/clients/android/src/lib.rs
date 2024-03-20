// The "system" ABI is only needed for Java FFI on Win32, not Android:
// https://github.com/jni-rs/jni-rs/pull/22
// However, this consideration has made it idiomatic for Java FFI in the Rust
// ecosystem, so it's used here for consistency.

use connlib_client_shared::{
    file_logger, keypair, Callbacks, Cidrv4, Cidrv6, Error, LoginUrl, LoginUrlError,
    ResourceDescription, Session,
};
use firezone_tunnel::Tun;
use jni::{
    objects::{GlobalRef, JByteArray, JClass, JObject, JObjectArray, JString, JValue, JValueGen},
    strings::JNIString,
    JNIEnv, JavaVM,
};
use secrecy::SecretString;
use std::{io, net::IpAddr, path::Path};
use std::{
    net::{Ipv4Addr, Ipv6Addr},
    path::PathBuf,
};
use std::{sync::OnceLock, time::Duration};
use thiserror::Error;
use tokio::runtime::Runtime;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

/// The Android client doesn't use platform APIs to detect network connectivity changes,
/// so we rely on connlib to do so. We have valid use cases for headless Android clients
/// (IoT devices, point-of-sale devices, etc), so try to reconnect for 30 days.
const MAX_PARTITION_TIME: Duration = Duration::from_secs(60 * 60 * 24 * 30);

pub struct CallbackHandler {
    vm: JavaVM,
    callback_handler: GlobalRef,
    handle: file_logger::Handle,
    new_tun_sender: tokio::sync::mpsc::Sender<Tun>,
}

impl Clone for CallbackHandler {
    fn clone(&self) -> Self {
        // This is essentially a `memcpy` to bypass redundant checks from
        // doing `as_raw` -> `from_raw`/etc; both of these fields are just
        // dumb pointers but the wrappers don't implement `Clone`.
        //
        // SAFETY: `self` is guaranteed to be valid and `Self` is POD.
        Self {
            vm: unsafe { std::ptr::read(&self.vm) },
            callback_handler: self.callback_handler.clone(),
            handle: self.handle.clone(),
            new_tun_sender: self.new_tun_sender.clone(),
        }
    }
}

#[derive(Debug, Error)]
pub enum CallbackError {
    #[error("Failed to attach current thread: {0}")]
    AttachCurrentThreadFailed(#[source] jni::errors::Error),
    #[error("Failed to serialize JSON: {0}")]
    SerializeFailed(#[from] serde_json::Error),
    #[error("Failed to create string `{name}`: {source}")]
    NewStringFailed {
        name: &'static str,
        source: jni::errors::Error,
    },
    #[error("Failed to call method `{name}`: {source}")]
    CallMethodFailed {
        name: &'static str,
        source: jni::errors::Error,
    },
}

impl CallbackHandler {
    fn env<T>(
        &self,
        f: impl FnOnce(JNIEnv) -> Result<T, CallbackError>,
    ) -> Result<T, CallbackError> {
        self.vm
            .attach_current_thread_as_daemon()
            .map_err(CallbackError::AttachCurrentThreadFailed)
            .and_then(f)
    }
}

fn call_method(
    env: &mut JNIEnv,
    this: &JObject,
    name: &'static str,
    sig: &str,
    args: &[JValue],
) -> Result<(), CallbackError> {
    env.call_method(this, name, sig, args)
        .map(|val| log::trace!("`{name}` returned `{val:?}`"))
        .map_err(|source| CallbackError::CallMethodFailed { name, source })
}

#[cfg(target_os = "android")]
fn android_layer<S>() -> impl tracing_subscriber::Layer<S>
where
    S: tracing::Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span>,
{
    tracing_android::layer("connlib").unwrap()
}

#[cfg(not(target_os = "android"))]
fn android_layer<S>() -> impl tracing_subscriber::Layer<S>
where
    S: tracing::Subscriber,
{
    tracing_subscriber::layer::Identity::new()
}

fn init_logging(log_dir: &Path, log_filter: String) -> file_logger::Handle {
    // On Android, logging state is persisted indefinitely after the System.loadLibrary
    // call, which means that a disconnect and tunnel process restart will not
    // reinitialize the guard. This is a problem because the guard remains tied to
    // the original process, which means that log events will not be rewritten to the log
    // file after a disconnect and reconnect.
    //
    // So we use a static variable to track whether the guard has been initialized and avoid
    // re-initialized it if so.
    static LOGGING_HANDLE: OnceLock<file_logger::Handle> = OnceLock::new();
    if let Some(handle) = LOGGING_HANDLE.get() {
        return handle.clone();
    }

    let (file_layer, handle) = file_logger::layer(log_dir);

    LOGGING_HANDLE
        .set(handle.clone())
        .expect("Logging guard should never be initialized twice");

    let _ = tracing_subscriber::registry()
        .with(file_layer.with_filter(EnvFilter::new(log_filter.clone())))
        .with(android_layer().with_filter(EnvFilter::new(log_filter.clone())))
        .try_init();

    handle
}

impl Callbacks for CallbackHandler {
    fn on_set_interface_config(
        &self,
        tunnel_address_v4: Ipv4Addr,
        tunnel_address_v6: Ipv6Addr,
        dns_addresses: Vec<IpAddr>,
    ) {
        let new_fd = self
            .env(|mut env| {
                let tunnel_address_v4 =
                    env.new_string(tunnel_address_v4.to_string())
                        .map_err(|source| CallbackError::NewStringFailed {
                            name: "tunnel_address_v4",
                            source,
                        })?;
                let tunnel_address_v6 =
                    env.new_string(tunnel_address_v6.to_string())
                        .map_err(|source| CallbackError::NewStringFailed {
                            name: "tunnel_address_v6",
                            source,
                        })?;
                let dns_addresses = env
                    .new_string(serde_json::to_string(&dns_addresses)?)
                    .map_err(|source| CallbackError::NewStringFailed {
                        name: "dns_addresses",
                        source,
                    })?;
                let name = "onSetInterfaceConfig";
                env.call_method(
                    &self.callback_handler,
                    name,
                    "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)I",
                    &[
                        JValue::from(&tunnel_address_v4),
                        JValue::from(&tunnel_address_v6),
                        JValue::from(&dns_addresses),
                    ],
                )
                .and_then(|val| val.i())
                .map_err(|source| CallbackError::CallMethodFailed { name, source })
            })
            .expect("onSetInterfaceConfig callback failed");

        let tun = match Tun::new(new_fd) {
            Ok(tun) => tun,
            Err(e) => {
                tracing::error!("Failed to make new TUN device");
                return;
            }
        };

        let _ = self.new_tun_sender.try_send(tun);

        // TODO: Make new `Tun` from new file descriptor and re-initialize it on the Tunnel.
    }

    fn on_tunnel_ready(&self) {
        self.env(|mut env| {
            call_method(
                &mut env,
                &self.callback_handler,
                "onTunnelReady",
                "()Z",
                &[],
            )
        })
        .expect("onTunnelReady callback failed")
    }

    fn on_update_routes(&self, route_list_4: Vec<Cidrv4>, route_list_6: Vec<Cidrv6>) {
        let new_fd = self
            .env(|mut env| {
                let route_list_4 = env
                    .new_string(serde_json::to_string(&route_list_4)?)
                    .map_err(|source| CallbackError::NewStringFailed {
                        name: "route_list_4",
                        source,
                    })?;
                let route_list_6 = env
                    .new_string(serde_json::to_string(&route_list_6)?)
                    .map_err(|source| CallbackError::NewStringFailed {
                        name: "route_list_6",
                        source,
                    })?;

                let name = "onUpdateRoutes";
                env.call_method(
                    &self.callback_handler,
                    name,
                    "(Ljava/lang/String;Ljava/lang/String;)I",
                    &[JValue::from(&route_list_4), JValue::from(&route_list_6)],
                )
                .and_then(|val| val.i())
                .map_err(|source| CallbackError::CallMethodFailed { name, source })
            })
            .expect("onUpdateRoutes callback failed");

        let tun = match Tun::new(new_fd) {
            Ok(tun) => tun,
            Err(e) => {
                tracing::error!("Failed to make new TUN device");
                return;
            }
        };

        let _ = self.new_tun_sender.try_send(
    }

    #[cfg(target_os = "android")]
    fn protect_file_descriptor(&self, file_descriptor: RawFd) {
        self.env(|mut env| {
            call_method(
                &mut env,
                &self.callback_handler,
                "protectFileDescriptor",
                "(I)V",
                &[JValue::Int(file_descriptor)],
            )
        })
        .expect("protectFileDescriptor callback failed");
    }

    fn on_update_resources(&self, resource_list: Vec<ResourceDescription>) {
        self.env(|mut env| {
            let resource_list = env
                .new_string(serde_json::to_string(&resource_list)?)
                .map_err(|source| CallbackError::NewStringFailed {
                    name: "resource_list",
                    source,
                })?;
            call_method(
                &mut env,
                &self.callback_handler,
                "onUpdateResources",
                "(Ljava/lang/String;)V",
                &[JValue::from(&resource_list)],
            )
        })
        .expect("onUpdateResources callback failed")
    }

    fn on_disconnect(&self, error: &Error) {
        self.env(|mut env| {
            let error = env
                .new_string(serde_json::to_string(&error.to_string())?)
                .map_err(|source| CallbackError::NewStringFailed {
                    name: "error",
                    source,
                })?;
            call_method(
                &mut env,
                &self.callback_handler,
                "onDisconnect",
                "(Ljava/lang/String;)Z",
                &[JValue::from(&error)],
            )
        })
        .expect("onDisconnect callback failed")
    }

    fn roll_log_file(&self) -> Option<PathBuf> {
        self.handle.roll_to_new_file().unwrap_or_else(|e| {
            tracing::debug!("Failed to roll over to new file: {e}");

            None
        })
    }

    fn get_system_default_resolvers(&self) -> Option<Vec<IpAddr>> {
        self.env(|mut env| {
            let name = "getSystemDefaultResolvers";
            let addrs = env
                .call_method(&self.callback_handler, name, "()[[B", &[])
                .and_then(JValueGen::l)
                .and_then(|arr| convert_byte_array_array(&mut env, arr.into()))
                .map_err(|source| CallbackError::CallMethodFailed { name, source })?;

            Ok(Some(addrs.iter().filter_map(|v| to_ip(v)).collect()))
        })
        .expect("getSystemDefaultResolvers callback failed")
    }
}

fn to_ip(val: &[u8]) -> Option<IpAddr> {
    let addr: Option<[u8; 4]> = val.try_into().ok();
    if let Some(addr) = addr {
        return Some(addr.into());
    }

    let addr: [u8; 16] = val.try_into().ok()?;
    Some(addr.into())
}

fn convert_byte_array_array(
    env: &mut JNIEnv,
    array: JObjectArray,
) -> jni::errors::Result<Vec<Vec<u8>>> {
    let len = env.get_array_length(&array)?;
    let mut result = Vec::with_capacity(len as usize);
    for i in 0..len {
        let arr: JByteArray<'_> = env.get_object_array_element(&array, i)?.into();
        result.push(env.convert_byte_array(arr)?);
    }
    Ok(result)
}

fn throw(env: &mut JNIEnv, class: &str, msg: impl Into<JNIString>) {
    if let Err(err) = env.throw_new(class, msg) {
        // We can't panic, since unwinding across the FFI boundary is UB...
        tracing::error!(?err, "failed to throw Java exception");
    }
}

fn catch_and_throw<F: FnOnce(&mut JNIEnv) -> R, R>(env: &mut JNIEnv, f: F) -> Option<R> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(env)))
        .map_err(|info| {
            tracing::error!("catching Rust panic");
            throw(
                env,
                "java/lang/Exception",
                match info.downcast_ref::<&str>() {
                    Some(msg) => format!("Rust panicked: {msg}"),
                    None => "Rust panicked with no message".to_owned(),
                },
            );
        })
        .ok()
}

#[derive(Debug, Error)]
enum ConnectError {
    #[error("Failed to access {name:?}: {source}")]
    StringInvalid {
        name: &'static str,
        source: jni::errors::Error,
    },
    #[error("Failed to get Java VM: {0}")]
    GetJavaVmFailed(#[source] jni::errors::Error),
    #[error(transparent)]
    ConnectFailed(#[from] Error),
    #[error(transparent)]
    InvalidLoginUrl(#[from] LoginUrlError<url::ParseError>),
    #[error("Unable to create tokio runtime: {0}")]
    UnableToCreateRuntime(#[from] io::Error),
}

macro_rules! string_from_jstring {
    ($env:expr, $j:ident) => {
        String::from(
            ($env)
                .get_string(&($j))
                .map_err(|source| ConnectError::StringInvalid {
                    name: stringify!($j),
                    source,
                })?,
        )
    };
}

// TODO: Refactor this when we refactor PhoenixChannel.
// See https://github.com/firezone/firezone/issues/2158
#[allow(clippy::too_many_arguments)]
fn connect(
    env: &mut JNIEnv,
    api_url: JString,
    token: JString,
    device_id: JString,
    device_name: JString,
    os_version: JString,
    log_dir: JString,
    log_filter: JString,
    callback_handler: GlobalRef,
) -> Result<SessionWrapper, ConnectError> {
    let api_url = string_from_jstring!(env, api_url);
    let secret = SecretString::from(string_from_jstring!(env, token));
    let device_id = string_from_jstring!(env, device_id);
    let device_name = string_from_jstring!(env, device_name);
    let os_version = string_from_jstring!(env, os_version);
    let log_dir = string_from_jstring!(env, log_dir);
    let log_filter = string_from_jstring!(env, log_filter);

    let (new_tun_sender, mut new_tun_receiver) = tokio::sync::mpsc::channel(1);

    let handle = init_logging(&PathBuf::from(log_dir), log_filter);

    let callback_handler = CallbackHandler {
        vm: env.get_java_vm().map_err(ConnectError::GetJavaVmFailed)?,
        callback_handler,
        handle,
        new_tun_sender,
    };

    let (private_key, public_key) = keypair();
    let login = LoginUrl::client(
        api_url.as_str(),
        &secret,
        device_id,
        Some(device_name),
        public_key.to_bytes(),
    )?;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .thread_name("connlib")
        .enable_all()
        .build()?;

    let session = Session::connect(
        login,
        private_key,
        Some(os_version),
        callback_handler,
        Some(MAX_PARTITION_TIME),
        runtime.handle().clone(),
    )?;

    {
        let mut session = session.clone();
        tokio::spawn(async move {
            while let Some(tun) = new_tun_receiver.recv().await {
                session.update_tun(tun);
            }
        });
    }

    Ok(SessionWrapper {
        inner: session,
        runtime,
    })
}

/// # Safety
/// Pointers must be valid
/// fd must be a valid file descriptor
#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "system" fn Java_dev_firezone_android_tunnel_ConnlibSession_connect(
    mut env: JNIEnv,
    _class: JClass,
    api_url: JString,
    token: JString,
    device_id: JString,
    device_name: JString,
    os_version: JString,
    log_dir: JString,
    log_filter: JString,
    callback_handler: JObject,
) -> *const SessionWrapper {
    let Ok(callback_handler) = env.new_global_ref(callback_handler) else {
        return std::ptr::null();
    };

    let connect = catch_and_throw(&mut env, |env| {
        connect(
            env,
            api_url,
            token,
            device_id,
            device_name,
            os_version,
            log_dir,
            log_filter,
            callback_handler,
        )
    });

    let session = match connect {
        Some(Ok(session)) => session,
        Some(Err(err)) => {
            throw(&mut env, "java/lang/Exception", err.to_string());
            return std::ptr::null();
        }
        None => return std::ptr::null(),
    };

    Box::into_raw(Box::new(session))
}

pub struct SessionWrapper {
    inner: Session,

    #[allow(dead_code)] // Only here so we don't drop the memory early.
    runtime: Runtime,
}

/// # Safety
/// Pointers must be valid
#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "system" fn Java_dev_firezone_android_tunnel_ConnlibSession_disconnect(
    mut env: JNIEnv,
    _: JClass,
    session: *mut SessionWrapper,
) {
    catch_and_throw(&mut env, |_| {
        Box::from_raw(session).inner.disconnect();
    });
}
