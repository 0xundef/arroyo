-- Arroyo pipeline: subscribe to LayerZero EndpointV2 PacketDelivered via ChainStack WSS.
-- Run with:  ./arroyo run -s ./state lz_pipeline.sql
-- Each WS text frame = one JSON-RPC object; we filter out the subscribe-confirmation frame.

CREATE TABLE lz_logs (
  jsonrpc TEXT,
  id INT,                                  -- present only in the subscribe-confirmation frame
  method TEXT,                             -- 'eth_subscription' in real notifications
  params STRUCT<
    subscription TEXT,
    result STRUCT<
      address TEXT,
      topics TEXT[],
      data TEXT,
      blockNumber TEXT,
      transactionHash TEXT,
      transactionIndex TEXT,
      blockHash TEXT,
      logIndex TEXT,
      removed BOOLEAN
    >
  >
) WITH (
  connector = 'websocket',
  endpoint = 'wss://ethereum-mainnet.core.chainstack.com/319d0f292df430b291b2bf0c0338dc8d',
  subscription_message = '{"jsonrpc":"2.0","id":1,"method":"eth_subscribe","params":["logs",{"address":"0x1a44076050125825900e736c501f859c50fe728c","topics":["0x3cd5e48f9730b129dc7550f0fcea9c767b7be37837cd10e55eb35f734f4bca04"]}]}',
  format = 'json'
);

-- Filter the confirmation frame, project raw fields, sink to stdout.
-- camelCase fields need quoted identifiers or DataFusion lowercases them.
-- blockNumber/logIndex are hex strings (0x…); keep as TEXT here — hex→int
-- + ABI decode is Step 2b (the decode_packet_delivered UDF).
SELECT
  params.result."blockNumber"     AS block_number,
  params.result."transactionHash" AS tx_hash,
  params.result."logIndex"        AS log_index,
  params.result.removed           AS removed,
  params.result.data              AS data
FROM lz_logs
WHERE method = 'eth_subscription';
