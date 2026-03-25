# FFI Usage

The library exposes C-style FFI functions for use from other languages.

## API Functions

| Function | Description |
|----------|-------------|
| `tracker_init(db_path)` | Create a tracker instance |
| `tracker_free(handle)` | Free a tracker instance |
| `tracker_request(handle, json_request)` | Send request, get response |
| `tracker_response_free(response)` | Free response string |
| `tracker_handle_json(json_request, db_path)` | One-shot JSON in/out |
| `tracker_version()` | Get library version |

## Request Format

```json
{"tool": "<tool_name>", "args": <args_object>}
```

## Response Format

```json
{
  "success": true/false,
  "data": <result_object>,
  "error": "error_message"
}
```

## Tools

### Transactions
- `create_transaction` - args: `{amount, kind, description}`
- `get_transaction` - args: `{id}`
- `list_transactions` - args: `{kind?}`
- `update_transaction` - args: `{id, amount?, kind?, description?}`
- `delete_transaction` - args: `{id}`

### Activities
- `create_activity` - args: `{start_time, stop_time, description}`
- `get_activity` - args: `{id}`
- `list_activities` - args: `null`
- `update_activity` - args: `{id, start_time?, stop_time?, description?}`
- `delete_activity` - args: `{id}`

## Example (C)

```c
TrackerHandle* handle = tracker_init(NULL);
char* response = tracker_request(handle, 
    "{\"tool\":\"list_transactions\",\"args\":null}");
printf("%s\n", response);
tracker_response_free(response);
tracker_free(handle);
```

## Example (Python)

```python
import ctypes
lib = ctypes.CDLL("./tx_tracker.dll")
response = lib.tracker_handle_json(
    b'{"tool":"list_transactions","args":null}', None)
```

## Building

```bash
# Release build
cargo build --release

# Output:
# - Windows: target/release/tx_tracker.dll
# - Linux: target/release/libtx_tracker.so
# - macOS: target/release/libtx_tracker.dylib
```
