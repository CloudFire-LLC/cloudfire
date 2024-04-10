use boringtun::x25519::{PublicKey, StaticSecret};
use firezone_relay::{AllocationPort, ChannelData, ClientSocket, IpStack, PeerSocket};
use rand::rngs::OsRng;
use snownet::{Answer, ClientNode, Event, MutableIpPacket, ServerNode, Transmit};
use std::{
    collections::HashSet,
    iter,
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4},
    time::{Duration, Instant, SystemTime},
    vec,
};
use str0m::{net::Protocol, Candidate};
use tracing::{debug_span, info_span, Span};

#[test]
fn smoke_direct() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_env_filter("debug")
        .try_init();

    let (alice, bob) = alice_and_bob();

    let mut alice =
        TestNode::new(info_span!("Alice"), alice, "1.1.1.1:80").with_local_host_candidate();

    let mut bob = TestNode::new(info_span!("Bob"), bob, "1.1.1.2:80").with_local_host_candidate();

    handshake(&mut alice, &mut bob, None);

    loop {
        if alice.is_connected_to(&bob) && bob.is_connected_to(&alice) {
            break;
        }
        progress(&mut alice, &mut bob, None);
    }
}

#[test]
fn smoke_relayed() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_env_filter("debug")
        .try_init();

    let (alice, bob) = alice_and_bob();

    let mut relay = TestRelay::new(IpAddr::V4(Ipv4Addr::LOCALHOST), debug_span!("Roger"));
    let mut alice = TestNode::new(debug_span!("Alice"), alice, "1.1.1.1:80")
        .with_drop_traffic_from("2.2.2.2:80");
    let mut bob =
        TestNode::new(debug_span!("Bob"), bob, "2.2.2.2:80").with_drop_traffic_from("1.1.1.1:80");

    handshake(&mut alice, &mut bob, Some(&relay));

    loop {
        if alice.is_connected_to(&bob) && bob.is_connected_to(&alice) {
            break;
        }
        progress(&mut alice, &mut bob, Some(&mut relay));
    }
}

#[test]
fn connection_times_out_after_20_seconds() {
    let (mut alice, _) = alice_and_bob();

    let created_at = Instant::now();

    let _ = alice.new_connection(
        1,
        HashSet::new(),
        HashSet::new(),
        Instant::now(),
        created_at,
    );
    alice.handle_timeout(created_at + Duration::from_secs(20));

    assert_eq!(alice.poll_event().unwrap(), Event::ConnectionFailed(1));
}

#[test]
fn connection_without_candidates_times_out_after_10_seconds() {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    let start = Instant::now();

    let (mut alice, mut bob) = alice_and_bob();
    let answer = send_offer(&mut alice, &mut bob, start);

    let accepted_at = start + Duration::from_secs(1);
    alice.accept_answer(1, bob.public_key(), answer, accepted_at);

    alice.handle_timeout(accepted_at + Duration::from_secs(10));

    assert_eq!(alice.poll_event().unwrap(), Event::ConnectionFailed(1));
}

#[test]
fn connection_with_candidates_does_not_time_out_after_10_seconds() {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    let start = Instant::now();

    let (mut alice, mut bob) = alice_and_bob();
    let answer = send_offer(&mut alice, &mut bob, start);

    let accepted_at = start + Duration::from_secs(1);
    alice.accept_answer(1, bob.public_key(), answer, accepted_at);
    alice.add_local_host_candidate(s("10.0.0.2:4444")).unwrap();
    alice.add_remote_candidate(1, host("10.0.0.1:4444"), accepted_at);

    alice.handle_timeout(accepted_at + Duration::from_secs(10));

    let any_failed =
        iter::from_fn(|| alice.poll_event()).any(|e| matches!(e, Event::ConnectionFailed(_)));

    assert!(!any_failed);
}

#[test]
fn answer_after_stale_connection_does_not_panic() {
    let start = Instant::now();

    let (mut alice, mut bob) = alice_and_bob();
    let answer = send_offer(&mut alice, &mut bob, start);

    let now = start + Duration::from_secs(10);
    alice.handle_timeout(now);

    alice.accept_answer(1, bob.public_key(), answer, now + Duration::from_secs(1));
}

#[test]
fn only_generate_candidate_event_after_answer() {
    let local_candidate = SocketAddr::new(IpAddr::from(Ipv4Addr::LOCALHOST), 10000);

    let mut alice = ClientNode::<u64>::new(StaticSecret::random_from_rng(rand::thread_rng()));

    alice.add_local_host_candidate(local_candidate).unwrap();

    let mut bob = ServerNode::<u64>::new(StaticSecret::random_from_rng(rand::thread_rng()));

    let offer = alice.new_connection(
        1,
        HashSet::new(),
        HashSet::new(),
        Instant::now(),
        Instant::now(),
    );

    assert_eq!(
        alice.poll_event(),
        None,
        "no event to be emitted before accepting the answer"
    );

    let answer = bob.accept_connection(
        1,
        offer,
        alice.public_key(),
        HashSet::new(),
        HashSet::new(),
        Instant::now(),
    );

    alice.accept_answer(1, bob.public_key(), answer, Instant::now());

    assert!(iter::from_fn(|| alice.poll_event()).any(|ev| ev
        == Event::SignalIceCandidate {
            connection: 1,
            candidate: Candidate::host(local_candidate, Protocol::Udp)
                .unwrap()
                .to_sdp_string()
        }));
}

#[test]
fn second_connection_with_same_relay_reuses_allocation() {
    let mut alice = ClientNode::<u64>::new(StaticSecret::random_from_rng(rand::thread_rng()));

    let _ = alice.new_connection(
        1,
        HashSet::new(),
        HashSet::from([relay("user1", "pass1", "realm1")]),
        Instant::now(),
        Instant::now(),
    );

    let transmit = alice.poll_transmit().unwrap();
    assert_eq!(transmit.dst, RELAY);
    assert!(alice.poll_transmit().is_none());

    let _ = alice.new_connection(
        2,
        HashSet::new(),
        HashSet::from([relay("user1", "pass1", "realm1")]),
        Instant::now(),
        Instant::now(),
    );

    assert!(alice.poll_transmit().is_none());
}

fn alice_and_bob() -> (ClientNode<u64>, ServerNode<u64>) {
    let alice = ClientNode::<u64>::new(StaticSecret::random_from_rng(rand::thread_rng()));
    let bob = ServerNode::<u64>::new(StaticSecret::random_from_rng(rand::thread_rng()));

    (alice, bob)
}

fn send_offer(alice: &mut ClientNode<u64>, bob: &mut ServerNode<u64>, now: Instant) -> Answer {
    let offer = alice.new_connection(1, HashSet::new(), HashSet::new(), Instant::now(), now);

    bob.accept_connection(
        1,
        offer,
        alice.public_key(),
        HashSet::new(),
        HashSet::new(),
        now,
    )
}

fn relay(username: &str, pass: &str, realm: &str) -> (SocketAddr, String, String, String) {
    (
        RELAY,
        username.to_owned(),
        pass.to_owned(),
        realm.to_owned(),
    )
}

fn host(socket: &str) -> String {
    Candidate::host(s(socket), Protocol::Udp)
        .unwrap()
        .to_sdp_string()
}

fn s(socket: &str) -> SocketAddr {
    socket.parse().unwrap()
}

const RELAY: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 10000));

// Heavily inspired by https://github.com/algesten/str0m/blob/7ed5143381cf095f7074689cc254b8c9e50d25c5/src/ice/mod.rs#L547-L647.
struct TestNode {
    node: EitherNode,
    span: Span,
    received_packets: Vec<MutableIpPacket<'static>>,
    progress_count: u64,
    time: Instant,
    local: SocketAddr,

    drop_traffic_from: HashSet<SocketAddr>,

    buffer: Box<[u8; 10_000]>,
}

struct TestRelay {
    inner: firezone_relay::Server<OsRng>,
    listen_addr: SocketAddr,
    span: Span,
}

impl TestRelay {
    fn new(local: IpAddr, span: Span) -> Self {
        let inner = firezone_relay::Server::new(IpStack::from(local), OsRng, 49152, 65535);

        Self {
            inner,
            listen_addr: SocketAddr::from((local, 3478)),
            span,
        }
    }

    fn handle_client_input(
        &mut self,
        trans: Transmit<'_>,
        sender: &mut TestNode,
        receiver: &mut TestNode,
    ) {
        if let Some((port, peer)) = self.span.in_scope(|| {
            self.inner.handle_client_input(
                &trans.payload,
                ClientSocket::new(sender.local),
                sender.time,
            )
        }) {
            let payload = ChannelData::parse(&trans.payload)
                .expect("valid ChannelData if we should relay it")
                .data()
                .to_vec();

            // Check if we need to relay to ourselves (from one allocation to another)
            if peer.into_socket().ip() == self.listen_addr.ip() {
                if let Some((client, channel)) =
                    self.inner.handle_peer_traffic(&payload, peer, port)
                {
                    assert_eq!(
                        client.into_socket(),
                        sender.local,
                        "only relays to the other party"
                    );

                    let mut msg = vec![0u8; payload.len() + 4];
                    msg[4..].copy_from_slice(&payload);
                    firezone_relay::ChannelData::encode_header_to_slice(
                        channel,
                        payload.len() as u16,
                        &mut msg[..4],
                    );

                    if let Some((_, packet)) = receiver
                        .span
                        .in_scope(|| {
                            receiver.node.decapsulate(
                                receiver.local,
                                self.listen_addr,
                                &msg,
                                receiver.time,
                                receiver.buffer.as_mut(),
                            )
                        })
                        .unwrap()
                    {
                        receiver.received_packets.push(packet.to_owned());
                    }
                }

                return;
            }

            let buffer = receiver.buffer.as_mut();

            if let Some((_, packet)) = receiver
                .span
                .in_scope(|| {
                    receiver.node.decapsulate(
                        receiver.local,
                        SocketAddr::from((self.listen_addr.ip(), port.value())),
                        &payload,
                        receiver.time,
                        buffer,
                    )
                })
                .unwrap()
            {
                receiver.received_packets.push(packet.to_owned());
            }
        }
    }

    fn drain_messages(&mut self, a1: &mut TestNode, a2: &mut TestNode) {
        while let Some(command) = self.inner.next_command() {
            match command {
                firezone_relay::Command::SendMessage { payload, recipient } => {
                    let recipient = if recipient.into_socket() == a1.local {
                        &mut *a1
                    } else if recipient.into_socket() == a2.local {
                        &mut *a2
                    } else {
                        panic!("Relay generated traffic for unknown client")
                    };

                    let buffer = recipient.buffer.as_mut();

                    if let Some((_, packet)) = recipient
                        .span
                        .in_scope(|| {
                            recipient.node.decapsulate(
                                recipient.local,
                                self.listen_addr,
                                &payload,
                                recipient.time,
                                buffer,
                            )
                        })
                        .unwrap()
                    {
                        recipient.received_packets.push(packet.to_owned());
                    }
                }
                firezone_relay::Command::CreateAllocation { .. }
                | firezone_relay::Command::FreeAllocation { .. } => {
                    // We ignore these because in our test we don't perform any IO.
                }
            }
        }
    }

    fn make_credentials(&self, username: &str) -> (String, String) {
        let expiry = SystemTime::now() + Duration::from_secs(60);

        let secs = expiry
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("expiry must be later than UNIX_EPOCH")
            .as_secs();

        let password =
            firezone_relay::auth::generate_password(self.inner.auth_secret(), expiry, username);

        (format!("{secs}:{username}"), password)
    }
}

enum EitherNode {
    Server(ServerNode<u64>),
    Client(ClientNode<u64>),
}

impl From<ClientNode<u64>> for EitherNode {
    fn from(value: ClientNode<u64>) -> Self {
        Self::Client(value)
    }
}

impl From<ServerNode<u64>> for EitherNode {
    fn from(value: ServerNode<u64>) -> Self {
        Self::Server(value)
    }
}

impl EitherNode {
    fn poll_transmit(&mut self) -> Option<Transmit> {
        match self {
            EitherNode::Client(n) => n.poll_transmit(),
            EitherNode::Server(n) => n.poll_transmit(),
        }
    }

    fn poll_event(&mut self) -> Option<Event<u64>> {
        match self {
            EitherNode::Client(n) => n.poll_event(),
            EitherNode::Server(n) => n.poll_event(),
        }
    }

    fn poll_timeout(&mut self) -> Option<Instant> {
        match self {
            EitherNode::Client(n) => n.poll_timeout(),
            EitherNode::Server(n) => n.poll_timeout(),
        }
    }

    fn add_remote_candidate(&mut self, id: u64, candidate: String, now: Instant) {
        match self {
            EitherNode::Client(n) => n.add_remote_candidate(id, candidate, now),
            EitherNode::Server(n) => n.add_remote_candidate(id, candidate, now),
        }
    }

    fn add_local_host_candidate(&mut self, socket: SocketAddr) {
        match self {
            EitherNode::Client(n) => n.add_local_host_candidate(socket).unwrap(),
            EitherNode::Server(n) => n.add_local_host_candidate(socket).unwrap(),
        }
    }

    fn is_connected_to(&self, key: PublicKey) -> bool {
        match self {
            EitherNode::Client(n) => n.is_connected_to(key),
            EitherNode::Server(n) => n.is_connected_to(key),
        }
    }

    fn public_key(&self) -> PublicKey {
        match self {
            EitherNode::Client(n) => n.public_key(),
            EitherNode::Server(n) => n.public_key(),
        }
    }

    fn as_client_mut(&mut self) -> Option<&mut ClientNode<u64>> {
        match self {
            EitherNode::Server(_) => None,
            EitherNode::Client(c) => Some(c),
        }
    }

    fn as_server_mut(&mut self) -> Option<&mut ServerNode<u64>> {
        match self {
            EitherNode::Server(s) => Some(s),
            EitherNode::Client(_) => None,
        }
    }

    fn decapsulate<'s>(
        &mut self,
        local: SocketAddr,
        from: SocketAddr,
        packet: &[u8],
        now: Instant,
        buffer: &'s mut [u8],
    ) -> Result<Option<(u64, MutableIpPacket<'s>)>, snownet::Error> {
        match self {
            EitherNode::Client(n) => n.decapsulate(local, from, packet, now, buffer),
            EitherNode::Server(n) => n.decapsulate(local, from, packet, now, buffer),
        }
    }

    fn handle_timeout(&mut self, now: Instant) {
        match self {
            EitherNode::Client(n) => n.handle_timeout(now),
            EitherNode::Server(n) => n.handle_timeout(now),
        }
    }
}

impl TestNode {
    pub fn new(span: Span, node: impl Into<EitherNode>, local: &str) -> Self {
        let now = Instant::now();
        TestNode {
            node: node.into(),
            span,
            progress_count: 0,
            time: now,
            received_packets: vec![],
            buffer: Box::new([0u8; 10_000]),
            local: local.parse().unwrap(),
            drop_traffic_from: HashSet::default(),
        }
    }

    fn is_connected_to(&self, other: &TestNode) -> bool {
        self.node.is_connected_to(other.node.public_key())
    }

    fn with_local_host_candidate(mut self) -> Self {
        self.span
            .in_scope(|| self.node.add_local_host_candidate(self.local));

        self
    }

    fn with_drop_traffic_from(mut self, other: &str) -> Self {
        self.drop_traffic_from.insert(other.parse().unwrap());

        self
    }
}

fn handshake(client: &mut TestNode, server: &mut TestNode, relay: Option<&TestRelay>) {
    let client_node = &mut client.node.as_client_mut().unwrap();
    let server_node = &mut server.node.as_server_mut().unwrap();

    let client_credentials = relay.map(|relay| {
        let (username, password) = relay.make_credentials("client");

        (relay.listen_addr, username, password, "firezone".to_owned())
    });
    let server_credentials = relay.map(|relay| {
        let (username, password) = relay.make_credentials("client");

        (relay.listen_addr, username, password, "firezone".to_owned())
    });

    let offer = client.span.in_scope(|| {
        client_node.new_connection(
            1,
            HashSet::default(),
            HashSet::from_iter(client_credentials),
            client.time,
            client.time,
        )
    });
    let answer = server.span.in_scope(|| {
        server_node.accept_connection(
            1,
            offer,
            client_node.public_key(),
            HashSet::default(),
            HashSet::from_iter(server_credentials),
            server.time,
        )
    });
    client
        .span
        .in_scope(|| client_node.accept_answer(1, server_node.public_key(), answer, client.time));
}

fn progress(a1: &mut TestNode, a2: &mut TestNode, mut r: Option<&mut TestRelay>) {
    if let Some(relay) = r.as_mut() {
        relay.drain_messages(a1, a2);
    }

    let (f, t) = if a1.progress_count % 2 == a2.progress_count % 2 {
        (a2, a1)
    } else {
        (a1, a2)
    };

    t.progress_count += 1;
    if t.progress_count > 100 {
        panic!("Test looped more than 100 times");
    }

    while let Some(v) = t.span.in_scope(|| t.node.poll_event()) {
        match v {
            Event::SignalIceCandidate {
                connection,
                candidate,
            } => f.node.add_remote_candidate(connection, candidate, f.time),
            Event::ConnectionEstablished(_) => {}
            Event::ConnectionFailed(_) => {}
        };
    }

    if let Some(trans) = f.span.in_scope(|| f.node.poll_transmit()) {
        let Some(src) = trans.src else {
            if let Some(relay) = r.as_mut() {
                assert_eq!(trans.dst.port(), 3478);

                relay.handle_client_input(trans.into_owned(), f, t);
            }

            return;
        };

        if let Some(relay) = r.as_mut() {
            if trans.dst.ip() == relay.listen_addr.ip() {
                if let Some((client, channel)) = relay.span.in_scope(|| {
                    relay.inner.handle_peer_traffic(
                        &trans.payload,
                        PeerSocket::new(f.local),
                        AllocationPort::new(trans.dst.port()),
                    )
                }) {
                    assert_eq!(
                        client.into_socket(),
                        f.local,
                        "only relays to the other party"
                    );

                    let mut msg = vec![0u8; trans.payload.len() + 4];
                    msg[4..].copy_from_slice(&trans.payload);
                    firezone_relay::ChannelData::encode_header_to_slice(
                        channel,
                        trans.payload.len() as u16,
                        &mut msg[..4],
                    );

                    if let Some((_, packet)) = t
                        .span
                        .in_scope(|| {
                            t.node.decapsulate(
                                t.local,
                                relay.listen_addr,
                                &msg,
                                t.time,
                                t.buffer.as_mut(),
                            )
                        })
                        .unwrap()
                    {
                        t.received_packets.push(packet.to_owned());
                    }
                }

                return;
            }
        }

        if t.drop_traffic_from.contains(&src) {
            t.span
                .in_scope(|| tracing::debug!(target: "test_harness", %src, "Dropping traffic"));
            return;
        }

        if let Some((_, packet)) = t
            .span
            .in_scope(|| {
                t.node
                    .decapsulate(trans.dst, src, &trans.payload, t.time, t.buffer.as_mut())
            })
            .unwrap()
        {
            t.received_packets.push(packet.to_owned())
        }
    } else {
        t.span.in_scope(|| t.node.handle_timeout(t.time));
    }

    let tim_f = f.span.in_scope(|| f.node.poll_timeout()).unwrap_or(f.time);
    f.time = tim_f;

    let tim_t = t.span.in_scope(|| t.node.poll_timeout()).unwrap_or(t.time);
    t.time = tim_t;
}
