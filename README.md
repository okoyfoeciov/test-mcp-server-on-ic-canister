### 1. Note

This MCP server only support `Streamable HTTP`.

This MCP server only support `tools` capability.

Stateless, non-response-verification at the moment!

### 2. Usage

```sh
dfx deploy --playground
```
Serve at `https://{DEPLOYED_CANISTER_ID}.raw.icp0.io/mcp`

*Example request*:
```sh
curl --request POST \
  --url https://${DEPLOYED_CANISTER_ID}.raw.icp0.io/mcp \
  --header 'Accept: application/json' \
  --header 'Content-Type: application/json' \
  --data '{
  "jsonrpc": "2.0",
  "id": 0,
  "method": "tools/list"
}'
````