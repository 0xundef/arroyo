/*
[dependencies]
hex = "0.4"
serde_json = "1"
*/

use arroyo_udf_plugin::udf;

/// Decodes a LayerZero EndpointV2 `PacketDelivered(Origin, address)` log `data` field.
///
/// ABI: `struct Origin { uint32 srcEid; bytes32 sender; uint64 nonce; }`
///      `event PacketDelivered(Origin origin, address receiver);`
/// `data` = 4 x 32-byte words: [srcEid | sender(bytes32) | nonce | receiver].
/// Only `topic[0]` is indexed; origin + receiver are non-indexed (all in `data`).
///
/// Returns JSON: { srcEid, srcChain, sender, nonce, receiver } or NULL on malformed input.
#[udf]
pub fn decode_packet_delivered(data: &str) -> Option<String> {
    let hex = data.strip_prefix("0x").or_else(|| data.strip_prefix("0X"))?;
    let bytes = hex::decode(hex).ok()?;
    if bytes.len() < 4 * 32 {
        return None;
    }
    let word = |i: usize| &bytes[i * 32..(i + 1) * 32];
    let take_u32 = |i: usize| u32::from_be_bytes(word(i)[28..32].try_into().unwrap());
    let take_addr = |i: usize| format!("0x{}", hex::encode(&word(i)[12..32])); // last 20 bytes
    let take_u64 = |i: usize| u64::from_be_bytes(word(i)[24..32].try_into().unwrap());

    let src_eid = take_u32(0);
    let sender = take_addr(1);
    let nonce = take_u64(2);
    let receiver = take_addr(3);

    let src_chain = match src_eid {
        30101 => "ethereum",
        30102 => "bnb-chain",
        30103 => "sepolia",
        30106 => "avalanche",
        30109 => "polygon",
        30110 => "arbitrum",
        30112 => "fantom",
        30145 => "gnosis",
        30181 => "mantle",
        30184 => "base",
        30280 => "sei",
        30362 => "berachain",
        30398 => "megaeth",
        _ => "unknown",
    };

    Some(
        serde_json::json!({
            "srcEid": src_eid,
            "srcChain": src_chain,
            "sender": sender,
            "nonce": nonce,
            "receiver": receiver,
        })
        .to_string(),
    )
}
