use verzola_proxy::tls::{
    classify_negotiated_group, parse_negotiated_group_metadata, NegotiatedGroupClassification,
    NegotiatedGroupMetadata,
};

#[test]
fn parses_and_classifies_handshake_metadata_fixtures() {
    let fixtures = [
        Fixture {
            raw: "negotiated_group=X25519",
            expected_group: Some("X25519"),
            expected_normalized: Some("x25519"),
            expected_classification: NegotiatedGroupClassification::Classical,
        },
        Fixture {
            raw: "group: secp256r1",
            expected_group: Some("secp256r1"),
            expected_normalized: Some("secp256r1"),
            expected_classification: NegotiatedGroupClassification::Classical,
        },
        Fixture {
            raw: "key_exchange_group = ffdhe3072",
            expected_group: Some("ffdhe3072"),
            expected_normalized: Some("ffdhe3072"),
            expected_classification: NegotiatedGroupClassification::Classical,
        },
        Fixture {
            raw: r#""negotiated_group": "X25519MLKEM768""#,
            expected_group: Some("X25519MLKEM768"),
            expected_normalized: Some("x25519mlkem768"),
            expected_classification: NegotiatedGroupClassification::Pq,
        },
        Fixture {
            raw: "tls_version=TLS1.3 cipher=TLS_AES_256_GCM_SHA384 negotiated_group=x25519_kyber768draft00",
            expected_group: Some("x25519_kyber768draft00"),
            expected_normalized: Some("x25519kyber768draft00"),
            expected_classification: NegotiatedGroupClassification::Pq,
        },
        Fixture {
            raw: "named_group X448",
            expected_group: Some("X448"),
            expected_normalized: Some("x448"),
            expected_classification: NegotiatedGroupClassification::Classical,
        },
        Fixture {
            raw: "negotiated_group=0x6399",
            expected_group: Some("0x6399"),
            expected_normalized: Some("0x6399"),
            expected_classification: NegotiatedGroupClassification::None,
        },
        Fixture {
            raw: "cipher=TLS_AES_128_GCM_SHA256",
            expected_group: None,
            expected_normalized: None,
            expected_classification: NegotiatedGroupClassification::None,
        },
        Fixture {
            raw: "",
            expected_group: None,
            expected_normalized: None,
            expected_classification: NegotiatedGroupClassification::None,
        },
    ];

    for fixture in fixtures {
        let parsed = parse_negotiated_group_metadata(fixture.raw);
        assert_fixture(parsed, fixture);
    }
}

#[test]
fn parser_is_conservative_for_unknown_or_placeholder_values() {
    let unknown = NegotiatedGroupMetadata::parse("group=unknown");
    assert_eq!(unknown.classification, NegotiatedGroupClassification::None);
    assert_eq!(unknown.normalized_group.as_deref(), Some("unknown"));

    let placeholder = NegotiatedGroupMetadata::parse("negotiated_group=none");
    assert_eq!(placeholder.classification, NegotiatedGroupClassification::None);
    assert_eq!(placeholder.normalized_group.as_deref(), Some("none"));
}

#[test]
fn direct_classifier_treats_hybrid_groups_as_pq() {
    assert_eq!(
        classify_negotiated_group(Some("X25519MLKEM768")),
        NegotiatedGroupClassification::Pq
    );
    assert_eq!(
        classify_negotiated_group(Some("x25519")),
        NegotiatedGroupClassification::Classical
    );
    assert_eq!(
        classify_negotiated_group(Some("0x6399")),
        NegotiatedGroupClassification::None
    );
    assert_eq!(
        classify_negotiated_group(None),
        NegotiatedGroupClassification::None
    );
}

#[derive(Clone, Copy)]
struct Fixture {
    raw: &'static str,
    expected_group: Option<&'static str>,
    expected_normalized: Option<&'static str>,
    expected_classification: NegotiatedGroupClassification,
}

fn assert_fixture(parsed: NegotiatedGroupMetadata, fixture: Fixture) {
    assert_eq!(parsed.raw, fixture.raw);
    assert_eq!(parsed.negotiated_group.as_deref(), fixture.expected_group);
    assert_eq!(parsed.normalized_group.as_deref(), fixture.expected_normalized);
    assert_eq!(parsed.classification, fixture.expected_classification);
}
