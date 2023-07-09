use crate::auth::{generate_password, split_username, systemtime_from_unix, FIREZONE};
use crate::server::channel_data::ChannelData;
use crate::server::UDP_TRANSPORT;
use crate::Attribute;
use bytecodec::DecodeExt;
use std::io;
use std::time::Duration;
use stun_codec::rfc5389::attributes::{ErrorCode, MessageIntegrity, Nonce, Username};
use stun_codec::rfc5389::errors::BadRequest;
use stun_codec::rfc5389::methods::BINDING;
use stun_codec::rfc5766::attributes::{
    ChannelNumber, Lifetime, RequestedTransport, XorPeerAddress,
};
use stun_codec::rfc5766::methods::{ALLOCATE, CHANNEL_BIND, CREATE_PERMISSION, REFRESH};
use stun_codec::{BrokenMessage, Message, MessageClass, TransactionId};
use uuid::Uuid;

/// The maximum lifetime of an allocation.
const MAX_ALLOCATION_LIFETIME: Duration = Duration::from_secs(3600);

/// The default lifetime of an allocation.
///
/// See <https://www.rfc-editor.org/rfc/rfc8656#name-allocations-2>.
const DEFAULT_ALLOCATION_LIFETIME: Duration = Duration::from_secs(600);

#[derive(Default)]
pub struct Decoder {
    stun_message_decoder: stun_codec::MessageDecoder<Attribute>,
}

impl Decoder {
    pub fn decode<'a>(
        &mut self,
        input: &'a [u8],
    ) -> Result<Result<ClientMessage<'a>, Message<Attribute>>, Error> {
        // De-multiplex as per <https://www.rfc-editor.org/rfc/rfc8656#name-channels-2>.
        match input.first() {
            Some(0..=3) => {
                let message = self.stun_message_decoder.decode_from_bytes(input)??;

                use MessageClass::*;
                match (message.method(), message.class()) {
                    (BINDING, Request) => Ok(Ok(ClientMessage::Binding(Binding::parse(&message)))),
                    (ALLOCATE, Request) => {
                        Ok(Allocate::parse(&message).map(ClientMessage::Allocate))
                    }
                    (REFRESH, Request) => Ok(Ok(ClientMessage::Refresh(Refresh::parse(&message)))),
                    (CHANNEL_BIND, Request) => {
                        Ok(ChannelBind::parse(&message).map(ClientMessage::ChannelBind))
                    }
                    (CREATE_PERMISSION, Request) => Ok(Ok(ClientMessage::CreatePermission(
                        CreatePermission::parse(&message),
                    ))),
                    (_, Request) => Ok(Err(bad_request(&message))),
                    (method, class) => {
                        Err(Error::DecodeStun(bytecodec::Error::from(io::Error::new(
                            io::ErrorKind::Unsupported,
                            format!(
                                "handling method {} and {class:?} is not implemented",
                                method.as_u16()
                            ),
                        ))))
                    }
                }
            }
            Some(64..=79) => Ok(Ok(ClientMessage::ChannelData(ChannelData::parse(input)?))),
            Some(other) => Err(Error::UnknownMessageType(*other)),
            None => Err(Error::Eof),
        }
    }
}

#[derive(derive_more::From)]
pub enum ClientMessage<'a> {
    ChannelData(ChannelData<'a>),
    Binding(Binding),
    Allocate(Allocate),
    Refresh(Refresh),
    ChannelBind(ChannelBind),
    CreatePermission(CreatePermission),
}

impl<'a> ClientMessage<'a> {
    pub fn transaction_id(&self) -> Option<TransactionId> {
        match self {
            ClientMessage::Binding(request) => Some(request.transaction_id),
            ClientMessage::Allocate(request) => Some(request.transaction_id),
            ClientMessage::Refresh(request) => Some(request.transaction_id),
            ClientMessage::ChannelBind(request) => Some(request.transaction_id),
            ClientMessage::CreatePermission(request) => Some(request.transaction_id),
            ClientMessage::ChannelData(_) => None,
        }
    }
}

#[derive(Debug)]
pub struct Binding {
    transaction_id: TransactionId,
}

impl Binding {
    pub fn new(transaction_id: TransactionId) -> Self {
        Self { transaction_id }
    }

    pub fn parse(message: &Message<Attribute>) -> Self {
        let transaction_id = message.transaction_id();

        Binding { transaction_id }
    }

    pub fn transaction_id(&self) -> TransactionId {
        self.transaction_id
    }
}

pub struct Allocate {
    transaction_id: TransactionId,
    message_integrity: Option<MessageIntegrity>,
    requested_transport: RequestedTransport,
    lifetime: Option<Lifetime>,
    username: Option<Username>,
    nonce: Option<Nonce>,
}

impl Allocate {
    pub fn new_authenticated_udp(
        transaction_id: TransactionId,
        lifetime: Option<Lifetime>,
        username: Username,
        relay_secret: &str,
        nonce: Uuid,
    ) -> Self {
        let requested_transport = RequestedTransport::new(UDP_TRANSPORT);
        let nonce = Nonce::new(nonce.as_hyphenated().to_string()).expect("len(uuid) < 128");

        let mut message =
            Message::<Attribute>::new(MessageClass::Request, ALLOCATE, transaction_id);
        message.add_attribute(requested_transport.clone().into());
        message.add_attribute(username.clone().into());
        message.add_attribute(nonce.clone().into());

        if let Some(lifetime) = &lifetime {
            message.add_attribute(lifetime.clone().into());
        }

        let (expiry, salt) = split_username(username.name()).expect("a valid username");
        let expiry_systemtime = systemtime_from_unix(expiry);

        let password = generate_password(relay_secret, expiry_systemtime, salt);

        let message_integrity =
            MessageIntegrity::new_long_term_credential(&message, &username, &FIREZONE, &password)
                .unwrap();

        Self {
            transaction_id,
            message_integrity: Some(message_integrity),
            requested_transport,
            lifetime,
            username: Some(username),
            nonce: Some(nonce),
        }
    }

    pub fn new_unauthenticated_udp(
        transaction_id: TransactionId,
        lifetime: Option<Lifetime>,
    ) -> Self {
        let requested_transport = RequestedTransport::new(UDP_TRANSPORT);

        let mut message =
            Message::<Attribute>::new(MessageClass::Request, ALLOCATE, transaction_id);
        message.add_attribute(requested_transport.clone().into());

        if let Some(lifetime) = &lifetime {
            message.add_attribute(lifetime.clone().into());
        }

        Self {
            transaction_id,
            message_integrity: None,
            requested_transport,
            lifetime,
            username: None,
            nonce: None,
        }
    }

    pub fn parse(message: &Message<Attribute>) -> Result<Self, Message<Attribute>> {
        let transaction_id = message.transaction_id();
        let message_integrity = message.get_attribute::<MessageIntegrity>().cloned();
        let nonce = message.get_attribute::<Nonce>().cloned();
        let requested_transport = message
            .get_attribute::<RequestedTransport>()
            .ok_or(bad_request(message))?
            .clone();
        let lifetime = message.get_attribute::<Lifetime>().cloned();
        let username = message.get_attribute::<Username>().cloned();

        Ok(Allocate {
            transaction_id,
            message_integrity,
            requested_transport,
            lifetime,
            username,
            nonce,
        })
    }

    pub fn transaction_id(&self) -> TransactionId {
        self.transaction_id
    }

    pub fn message_integrity(&self) -> Option<&MessageIntegrity> {
        self.message_integrity.as_ref()
    }

    pub fn requested_transport(&self) -> &RequestedTransport {
        &self.requested_transport
    }

    pub fn effective_lifetime(&self) -> Lifetime {
        compute_effective_lifetime(self.lifetime.as_ref())
    }

    pub fn username(&self) -> Option<&Username> {
        self.username.as_ref()
    }

    pub fn nonce(&self) -> Option<&Nonce> {
        self.nonce.as_ref()
    }
}

pub struct Refresh {
    transaction_id: TransactionId,
    message_integrity: Option<MessageIntegrity>,
    lifetime: Option<Lifetime>,
    username: Option<Username>,
    nonce: Option<Nonce>,
}

impl Refresh {
    pub fn new(
        transaction_id: TransactionId,
        lifetime: Option<Lifetime>,
        username: Username,
        relay_secret: &str,
        nonce: Uuid,
    ) -> Self {
        let nonce = Nonce::new(nonce.as_hyphenated().to_string()).expect("len(uuid) < 128");

        let mut message = Message::<Attribute>::new(MessageClass::Request, REFRESH, transaction_id);
        message.add_attribute(username.clone().into());
        message.add_attribute(nonce.clone().into());

        if let Some(lifetime) = &lifetime {
            message.add_attribute(lifetime.clone().into());
        }

        let (expiry, salt) = split_username(username.name()).expect("a valid username");
        let expiry_systemtime = systemtime_from_unix(expiry);

        let password = generate_password(relay_secret, expiry_systemtime, salt);

        let message_integrity =
            MessageIntegrity::new_long_term_credential(&message, &username, &FIREZONE, &password)
                .unwrap();

        Self {
            transaction_id,
            message_integrity: Some(message_integrity),
            lifetime,
            username: Some(username),
            nonce: Some(nonce),
        }
    }

    pub fn parse(message: &Message<Attribute>) -> Self {
        let transaction_id = message.transaction_id();
        let message_integrity = message.get_attribute::<MessageIntegrity>().cloned();
        let nonce = message.get_attribute::<Nonce>().cloned();
        let lifetime = message.get_attribute::<Lifetime>().cloned();
        let username = message.get_attribute::<Username>().cloned();

        Refresh {
            transaction_id,
            message_integrity,
            lifetime,
            username,
            nonce,
        }
    }

    pub fn transaction_id(&self) -> TransactionId {
        self.transaction_id
    }

    pub fn message_integrity(&self) -> Option<&MessageIntegrity> {
        self.message_integrity.as_ref()
    }

    pub fn effective_lifetime(&self) -> Lifetime {
        compute_effective_lifetime(self.lifetime.as_ref())
    }

    pub fn username(&self) -> Option<&Username> {
        self.username.as_ref()
    }

    pub fn nonce(&self) -> Option<&Nonce> {
        self.nonce.as_ref()
    }
}

pub struct ChannelBind {
    transaction_id: TransactionId,
    channel_number: ChannelNumber,
    message_integrity: Option<MessageIntegrity>,
    nonce: Option<Nonce>,
    xor_peer_address: XorPeerAddress,
    username: Option<Username>,
}

impl ChannelBind {
    pub fn new(
        transaction_id: TransactionId,
        channel_number: ChannelNumber,
        xor_peer_address: XorPeerAddress,
        username: Username,
        relay_secret: &str,
        nonce: Uuid,
    ) -> Self {
        let nonce = Nonce::new(nonce.as_hyphenated().to_string()).expect("len(uuid) < 128");

        let mut message =
            Message::<Attribute>::new(MessageClass::Request, CHANNEL_BIND, transaction_id);
        message.add_attribute(username.clone().into());
        message.add_attribute(channel_number.into());
        message.add_attribute(xor_peer_address.clone().into());
        message.add_attribute(nonce.clone().into());

        let (expiry, salt) = split_username(username.name()).expect("a valid username");
        let expiry_systemtime = systemtime_from_unix(expiry);

        let password = generate_password(relay_secret, expiry_systemtime, salt);

        let message_integrity =
            MessageIntegrity::new_long_term_credential(&message, &username, &FIREZONE, &password)
                .unwrap();

        Self {
            transaction_id,
            channel_number,
            message_integrity: Some(message_integrity),
            xor_peer_address,
            username: Some(username),
            nonce: Some(nonce),
        }
    }

    pub fn parse(message: &Message<Attribute>) -> Result<Self, Message<Attribute>> {
        let transaction_id = message.transaction_id();
        let channel_number = message
            .get_attribute::<ChannelNumber>()
            .copied()
            .ok_or(bad_request(message))?;
        let message_integrity = message.get_attribute::<MessageIntegrity>().cloned();
        let nonce = message.get_attribute::<Nonce>().cloned();
        let username = message.get_attribute::<Username>().cloned();
        let xor_peer_address = message
            .get_attribute::<XorPeerAddress>()
            .ok_or(bad_request(message))?
            .clone();

        Ok(ChannelBind {
            transaction_id,
            channel_number,
            message_integrity,
            nonce,
            xor_peer_address,
            username,
        })
    }

    pub fn transaction_id(&self) -> TransactionId {
        self.transaction_id
    }

    pub fn channel_number(&self) -> ChannelNumber {
        self.channel_number
    }

    pub fn message_integrity(&self) -> Option<&MessageIntegrity> {
        self.message_integrity.as_ref()
    }

    pub fn xor_peer_address(&self) -> &XorPeerAddress {
        &self.xor_peer_address
    }

    pub fn username(&self) -> Option<&Username> {
        self.username.as_ref()
    }

    pub fn nonce(&self) -> Option<&Nonce> {
        self.nonce.as_ref()
    }
}

pub struct CreatePermission {
    transaction_id: TransactionId,
    message_integrity: Option<MessageIntegrity>,
    username: Option<Username>,
    nonce: Option<Nonce>,
}

impl CreatePermission {
    pub fn parse(message: &Message<Attribute>) -> Self {
        let transaction_id = message.transaction_id();
        let message_integrity = message.get_attribute::<MessageIntegrity>().cloned();
        let username = message.get_attribute::<Username>().cloned();
        let nonce = message.get_attribute::<Nonce>().cloned();

        CreatePermission {
            transaction_id,
            message_integrity,
            username,
            nonce,
        }
    }

    pub fn transaction_id(&self) -> TransactionId {
        self.transaction_id
    }

    pub fn message_integrity(&self) -> Option<&MessageIntegrity> {
        self.message_integrity.as_ref()
    }

    pub fn username(&self) -> Option<&Username> {
        self.username.as_ref()
    }

    pub fn nonce(&self) -> Option<&Nonce> {
        self.nonce.as_ref()
    }
}

/// Computes the effective lifetime of an allocation.
fn compute_effective_lifetime(requested_lifetime: Option<&Lifetime>) -> Lifetime {
    let Some(requested) = requested_lifetime else {
        return Lifetime::new(DEFAULT_ALLOCATION_LIFETIME).unwrap();
    };

    let effective_lifetime = requested.lifetime().min(MAX_ALLOCATION_LIFETIME);

    Lifetime::new(effective_lifetime).unwrap()
}

fn bad_request(message: &Message<Attribute>) -> Message<Attribute> {
    let mut message = Message::new(
        MessageClass::ErrorResponse,
        message.method(),
        message.transaction_id(),
    );
    message.add_attribute(ErrorCode::from(BadRequest).into());

    message
}

#[derive(Debug)]
pub enum Error {
    BadChannelData(io::Error),
    DecodeStun(bytecodec::Error),
    UnknownMessageType(u8),
    Eof,
}

impl From<BrokenMessage> for Error {
    fn from(msg: BrokenMessage) -> Self {
        Error::DecodeStun(msg.into())
    }
}

impl From<bytecodec::Error> for Error {
    fn from(error: bytecodec::Error) -> Self {
        Error::DecodeStun(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::BadChannelData(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requested_lifetime_is_capped_at_max_lifetime() {
        let requested_lifetime = Lifetime::new(Duration::from_secs(10_000_000)).unwrap();

        let effective_lifetime = compute_effective_lifetime(Some(&requested_lifetime));

        assert_eq!(effective_lifetime.lifetime(), MAX_ALLOCATION_LIFETIME)
    }
}
