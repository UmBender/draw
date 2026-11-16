//! The selection decoders, the `INCR` sender and receiver and the target
//! tables, without a server (T177 AC-1, AC-3). The fuzz uses a seeded
//! xorshift generator: `proptest` is not a dev-dependency of this crate.

#![allow(clippy::unwrap_used)]

use draw_sys::x11::selection::{
    Answer, INCR_CHUNK, IncrReceiver, IncrSender, IncrStep, MAX_INCOMING, Payload, SelEvent,
    TargetKind, decode_selection,
};

const SELECTION_CLEAR: i32 = 29;
const SELECTION_REQUEST: i32 = 30;
const SELECTION_NOTIFY: i32 = 31;
const PROPERTY_NOTIFY: i32 = 28;

fn event_of(kind: i32) -> [u8; 192] {
    let mut raw = [0u8; 192];
    raw[0..4].copy_from_slice(&kind.to_ne_bytes());
    raw
}

fn put(raw: &mut [u8; 192], at: usize, value: u64) {
    raw[at..at + 8].copy_from_slice(&value.to_ne_bytes());
}

#[test]
fn decode_selection_request_reads_every_field() {
    let mut raw = event_of(SELECTION_REQUEST);
    put(&mut raw, 40, 11);
    put(&mut raw, 48, 22);
    put(&mut raw, 56, 33);
    put(&mut raw, 64, 44);
    put(&mut raw, 72, 55);

    let event = decode_selection(&raw).unwrap();

    let SelEvent::Request(r) = event else {
        panic!("not a request: {event:?}");
    };
    assert_eq!(
        (r.requestor, r.selection, r.target, r.property, r.time),
        (11, 22, 33, 44, 55)
    );
}

#[test]
fn decode_selection_notify_reads_target_and_property() {
    let mut raw = event_of(SELECTION_NOTIFY);
    put(&mut raw, 40, 22);
    put(&mut raw, 48, 33);
    put(&mut raw, 56, 0);

    let event = decode_selection(&raw).unwrap();

    assert_eq!(
        event,
        SelEvent::Notify {
            selection: 22,
            target: 33,
            property: 0
        }
    );
}

#[test]
fn decode_selection_clear_reads_the_selection() {
    let mut raw = event_of(SELECTION_CLEAR);
    put(&mut raw, 40, 9);

    assert_eq!(
        decode_selection(&raw),
        Some(SelEvent::Clear { selection: 9 })
    );
}

#[test]
fn decode_property_notify_tells_delete_from_new_value() {
    let mut raw = event_of(PROPERTY_NOTIFY);
    put(&mut raw, 32, 5);
    put(&mut raw, 40, 6);
    raw[56..60].copy_from_slice(&1i32.to_ne_bytes());

    assert_eq!(
        decode_selection(&raw),
        Some(SelEvent::Property {
            window: 5,
            atom: 6,
            deleted: true
        })
    );
    raw[56..60].copy_from_slice(&0i32.to_ne_bytes());
    assert_eq!(
        decode_selection(&raw),
        Some(SelEvent::Property {
            window: 5,
            atom: 6,
            deleted: false
        })
    );
}

#[test]
fn decode_selection_other_events_are_none() {
    assert_eq!(decode_selection(&event_of(2)), None);
    assert_eq!(decode_selection(&[]), None);
    assert_eq!(decode_selection(&event_of(SELECTION_REQUEST)[..20]), None);
}

#[test]
fn targets_for_text_list_text_targets_and_not_png() {
    let payload = Payload::Text("hi".to_owned());

    let Answer::Targets(list) = payload.answer(TargetKind::Targets) else {
        panic!("TARGETS must be answered with a list");
    };

    assert!(list.contains(&TargetKind::Targets));
    assert!(list.contains(&TargetKind::Utf8));
    assert!(list.contains(&TargetKind::String));
    assert!(!list.contains(&TargetKind::Png));
}

#[test]
fn targets_for_png_list_only_png_and_targets() {
    let payload = Payload::Png(vec![1, 2, 3]);

    let Answer::Targets(list) = payload.answer(TargetKind::Targets) else {
        panic!("TARGETS must be answered with a list");
    };

    assert_eq!(list, vec![TargetKind::Targets, TargetKind::Png]);
}

#[test]
fn targets_text_is_served_as_its_utf8_bytes() {
    let payload = Payload::Text("héllo".to_owned());

    for kind in [TargetKind::Utf8, TargetKind::String, TargetKind::Text] {
        assert_eq!(
            payload.answer(kind),
            Answer::Data("héllo".as_bytes().to_vec(), kind),
            "{kind:?}"
        );
    }
}

#[test]
fn targets_png_is_not_served_as_text_and_text_not_as_png() {
    let png = Payload::Png(vec![0x89, b'P']);
    let text = Payload::Text("x".to_owned());

    assert_eq!(png.answer(TargetKind::Utf8), Answer::Refuse);
    assert_eq!(text.answer(TargetKind::Png), Answer::Refuse);
    assert_eq!(text.answer(TargetKind::Other), Answer::Refuse);
}

#[test]
fn incr_sender_chunks_then_sends_an_empty_end_chunk() {
    let data = vec![7u8; INCR_CHUNK * 2 + 5];
    let mut sender = IncrSender::new(data.clone());

    let mut got = Vec::new();
    let mut chunks = 0;
    while let Some(chunk) = sender.next_chunk() {
        chunks += 1;
        assert!(chunk.len() <= INCR_CHUNK);
        if chunk.is_empty() {
            break;
        }
        got.extend_from_slice(chunk);
    }

    assert_eq!(got, data);
    assert_eq!(chunks, 4);
    assert!(sender.next_chunk().is_none());
}

#[test]
fn incr_sender_with_no_data_sends_only_the_end_chunk() {
    let mut sender = IncrSender::new(Vec::new());

    assert_eq!(sender.next_chunk(), Some(&[][..]));
    assert!(sender.next_chunk().is_none());
}

#[test]
fn incr_receiver_joins_chunks_until_the_empty_one() {
    let mut receiver = IncrReceiver::new();

    assert_eq!(receiver.push(b"ab"), IncrStep::More);
    assert_eq!(receiver.push(b"cd"), IncrStep::More);
    assert_eq!(receiver.push(b""), IncrStep::Done);

    assert_eq!(receiver.finish(), b"abcd");
}

#[test]
fn incr_receiver_overflows_past_the_cap() {
    let mut receiver = IncrReceiver::new();
    let block = vec![0u8; 1 << 20];

    let mut steps = Vec::new();
    for _ in 0..=(MAX_INCOMING >> 20) {
        steps.push(receiver.push(&block));
    }

    assert_eq!(steps.last(), Some(&IncrStep::Overflow));
    assert!(
        steps[..steps.len() - 1]
            .iter()
            .all(|s| *s == IncrStep::More)
    );
}

struct Xorshift(u64);

impl Xorshift {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

#[test]
fn selection_decoders_never_panic_on_random_bytes() {
    let mut rng = Xorshift(0x9E37_79B9_7F4A_7C15);
    for round in 0..5000 {
        let len = (rng.next() % 220) as usize;
        let mut bytes: Vec<u8> = (0..len).map(|_| rng.next() as u8).collect();
        if round % 2 == 0 && bytes.len() >= 4 {
            let kind = 28 + (rng.next() % 4) as i32;
            bytes[0..4].copy_from_slice(&kind.to_ne_bytes());
        }
        let _ = decode_selection(&bytes);
    }
}

#[test]
fn incr_receiver_never_exceeds_the_cap() {
    let mut rng = Xorshift(0xD1B5_4A32_D192_ED03);
    for _ in 0..20 {
        let mut receiver = IncrReceiver::new();
        for _ in 0..200 {
            let len = (rng.next() % (1 << 21)) as usize;
            let chunk = vec![1u8; len];
            if receiver.push(&chunk) != IncrStep::More {
                break;
            }
        }
        assert!(receiver.finish().len() <= MAX_INCOMING);
    }
}
