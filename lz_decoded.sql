-- Arroyo pipeline: poll ChainStack eth_getLogs(latest) for LayerZero PacketDelivered,
-- capture the WHOLE response body into a single TEXT column via format='raw_string'
-- (avoids array-of-struct / UNNEST entirely), then decode all logs in one UDF.
--
-- emit_behavior='changed' dedups identical responses (same block => same body => 1 emit).
-- One output row per block that contains >=1 PacketDelivered; `decoded` is a JSON array.

CREATE TABLE lz_raw ("value" TEXT) WITH (
  connector = 'polling_http',
  endpoint = 'https://ethereum-mainnet.core.chainstack.com/319d0f292df430b291b2bf0c0338dc8d',
  method = 'POST',
  headers = 'Content-Type: application/json',
  body = '{"jsonrpc":"2.0","id":1,"method":"eth_getLogs","params":[{"fromBlock":"latest","toBlock":"latest","address":"0x1a44076050125825900e736c501f859c50fe728c","topics":["0x3cd5e48f9730b129dc7550f0fcea9c767b7be37837cd10e55eb35f734f4bca04"]}]}',
  poll_interval_ms = '6000',
  emit_behavior = 'changed',
  format = 'raw_string'
);

SELECT decode_packets("value") AS decoded
FROM lz_raw;
