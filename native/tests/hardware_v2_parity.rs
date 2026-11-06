use cirnet_contracts_native::hardware_v2::{
    CANONICAL_SERVICE, FILE_DESCRIPTOR_SET, InstrumentCapability, LEGACY_FILE_DESCRIPTOR_SET,
    LEGACY_SERVICE, OperationReply,
};
use prost::Message;
use prost_types::{DescriptorProto, FileDescriptorProto, FileDescriptorSet};

fn single_file(payload: &[u8]) -> FileDescriptorProto {
    let mut files = FileDescriptorSet::decode(payload).unwrap().file;
    assert_eq!(files.len(), 1);
    files.pop().unwrap()
}

fn normalize_type_name(name: &mut Option<String>) {
    if let Some(value) = name {
        *value = value.replace(".netcircuit.hardware.v2.", ".cirnet.hardware.v2.");
    }
}

fn normalize_message(message: &mut DescriptorProto) {
    for field in &mut message.field {
        normalize_type_name(&mut field.type_name);
    }
    for nested in &mut message.nested_type {
        normalize_message(nested);
    }
}

#[test]
fn hardware_descriptors_preserve_both_names_and_wire_fields() {
    let canonical = single_file(FILE_DESCRIPTOR_SET);
    let mut legacy = single_file(LEGACY_FILE_DESCRIPTOR_SET);
    assert_eq!(canonical.service.len(), 1);
    assert_eq!(legacy.service.len(), 1);
    assert_eq!(
        format!("{}.{}", canonical.package(), canonical.service[0].name()),
        CANONICAL_SERVICE
    );
    assert_eq!(
        format!("{}.{}", legacy.package(), legacy.service[0].name()),
        LEGACY_SERVICE
    );
    assert_eq!(canonical.service[0].method.len(), 14);
    legacy.package.clone_from(&canonical.package);
    for message in &mut legacy.message_type {
        normalize_message(message);
    }
    for method in &mut legacy.service[0].method {
        normalize_type_name(&mut method.input_type);
        normalize_type_name(&mut method.output_type);
    }
    assert_eq!(legacy, canonical);
}

#[test]
fn protobuf_preserves_explicit_default_and_binary_capture_tag() {
    let absent = InstrumentCapability::default();
    let explicit = InstrumentCapability {
        status: Some(String::new()),
        ..Default::default()
    };
    assert!(absent.encode_to_vec().is_empty());
    assert_eq!(explicit.encode_to_vec(), [26, 0]);
    assert_eq!(
        InstrumentCapability::decode([26, 0].as_slice()).unwrap(),
        explicit
    );

    let reply = OperationReply {
        capture_payload: b"NCAP".to_vec(),
        ..Default::default()
    };
    assert_eq!(reply.encode_to_vec(), b"\x2a\x04NCAP");
}
