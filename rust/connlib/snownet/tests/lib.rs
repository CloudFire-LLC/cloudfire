use snownet::{Answer, ClientNode, Event, ServerNode};
use std::{
    iter,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::{Duration, Instant},
};
use str0m::{net::Protocol, Candidate};

#[test]
fn connection_times_out_after_20_seconds() {
    let (mut alice, _) = alice_and_bob();

    let created_at = Instant::now();

    let _ = alice.new_connection(1, Instant::now(), created_at);
    alice.handle_timeout(created_at + Duration::from_secs(20));

    assert_eq!(alice.poll_event().unwrap(), Event::ConnectionFailed(1));
}

#[test]
fn connection_without_candidates_times_out_after_10_seconds() {
    let _guard = firezone_logging::test("trace");
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
    let _guard = firezone_logging::test("trace");
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

    let mut alice = ClientNode::<u64, u64>::new(rand::random());
    alice.add_local_host_candidate(local_candidate).unwrap();

    let mut bob = ServerNode::<u64, u64>::new(rand::random());

    let offer = alice.new_connection(1, Instant::now(), Instant::now());

    assert_eq!(
        alice.poll_event(),
        None,
        "no event to be emitted before accepting the answer"
    );

    let answer = bob.accept_connection(1, offer, alice.public_key(), Instant::now());

    alice.accept_answer(1, bob.public_key(), answer, Instant::now());

    assert!(iter::from_fn(|| alice.poll_event()).any(|ev| ev
        == Event::NewIceCandidate {
            connection: 1,
            candidate: Candidate::host(local_candidate, Protocol::Udp)
                .unwrap()
                .to_sdp_string()
        }));
}

fn alice_and_bob() -> (ClientNode<u64, u64>, ServerNode<u64, u64>) {
    let alice = ClientNode::new(rand::random());
    let bob = ServerNode::new(rand::random());

    (alice, bob)
}

fn send_offer(
    alice: &mut ClientNode<u64, u64>,
    bob: &mut ServerNode<u64, u64>,
    now: Instant,
) -> Answer {
    let offer = alice.new_connection(1, Instant::now(), now);

    bob.accept_connection(1, offer, alice.public_key(), now)
}

fn host(socket: &str) -> String {
    Candidate::host(s(socket), Protocol::Udp)
        .unwrap()
        .to_sdp_string()
}

fn s(socket: &str) -> SocketAddr {
    socket.parse().unwrap()
}
