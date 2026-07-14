/*
[dependencies]
hex = "0.4"
serde_json = "1"
*/

use arroyo_udf_plugin::udf;

/// Decodes a whole ChainStack `eth_getLogs` response (captured verbatim into the
/// `value` TEXT column via `format='raw_string'`) into a JSON array of decoded
/// LayerZero EndpointV2 `PacketDelivered` events, one entry per log.
///
/// Input `value` looks like:
///   {"jsonrpc":"2.0","id":1,"result":[{ "address":.., "topics":[..],
///     "data":"0x<4 words>", "blockNumber":"0x..", "transactionHash":"0x..",
///     "logIndex":"0x..", ... }, ...]}
///
/// For each log, decodes PacketDelivered(origin, receiver) from `data`:
///   word0 = srcEid (u32), word1 = sender (bytes32), word2 = nonce (u64),
///   word3 = receiver (address). Returns [] when the response has no logs.
#[udf]
pub fn decode_packets(value: &str) -> Option<String> {
    let resp: serde_json::Value = serde_json::from_str(value).ok()?;
    let arr = resp.get("result")?.as_array()?;
    if arr.is_empty() {
        return Some("[]".to_string());
    }

    let mut out: Vec<serde_json::Value> = Vec::with_capacity(arr.len());
    for log in arr {
        let data = match log.get("data").and_then(|d| d.as_str()) {
            Some(d) => d,
            None => continue,
        };
        let block = log
            .get("blockNumber")
            .and_then(|b| b.as_str())
            .unwrap_or("");
        let tx = log
            .get("transactionHash")
            .and_then(|b| b.as_str())
            .unwrap_or("");
        let log_index = log.get("logIndex").and_then(|b| b.as_str()).unwrap_or("");

        let hex_str = data.strip_prefix("0x").or_else(|| data.strip_prefix("0X"));
        let bytes = match hex_str.and_then(|h| hex::decode(h).ok()) {
            Some(b) if b.len() >= 4 * 32 => b,
            _ => continue, // not a PacketDelivered data payload (or malformed)
        };
        let word = |i: usize| &bytes[i * 32..(i + 1) * 32];
        let src_eid = u32::from_be_bytes(word(0)[28..32].try_into().unwrap());
        let sender = format!("0x{}", hex::encode(&word(1)[12..32]));
        let nonce = u64::from_be_bytes(word(2)[24..32].try_into().unwrap());
        let receiver = format!("0x{}", hex::encode(&word(3)[12..32]));

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
            // Unmapped srcEid -> discard this log (per go-live policy:
            // only emit packets from chains we explicitly recognize).
            _ => {
                continue;
            }
        };

        out.push(serde_json::json!({
            "srcEid": src_eid,
            "srcChain": src_chain,
            "sender": sender,
            "nonce": nonce,
            "receiver": receiver,
            "blockNumber": block,
            "transactionHash": tx,
            "logIndex": log_index,
        }));
    }

    Some(serde_json::Value::Array(out).to_string())
}
