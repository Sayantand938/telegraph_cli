# logbook

A Rust library for tracking transactions and activities with SQLite storage. Provides both a native Rust API and a C-compatible FFI interface for integration with other languages.

## Features

- **Transaction Tracking**: Create, read, update, and delete financial transactions
- **Activity Logging**: Track time-based activities with start/stop times
- **Categorization**: Organize items by categories, places, tags, and persons
- **SQLite Storage**: Persistent local storage with automatic schema management
- **FFI Support**: C-compatible API for integration with C, Python, and other languages
- **Async Runtime**: Built on Tokio for high-performance async operations

## Installation

### As a Rust Library

Add to your `Cargo.toml`:

```toml
[dependencies]
logbook = { path = "path/to/logbook" }
```

### Build the Library

```bash
# Build the library
cargo build --release

# Build the DLL (Windows)
cargo build --release --lib
```

## Usage

### Rust API

```rust
use logbook::{Tracker, Request};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracker with default database path
    let tracker = Tracker::new(None).await?;

    // Create a transaction
    let response = tracker.handle(&Request {
        tool: "create_transaction".into(),
        args: json!({
            "amount": 50.0,
            "kind": "shopping",
            "description": "Groceries"
        }),
    }).await;

    println!("Success: {}", response.success);
    println!("Message: {}", response.message.unwrap_or_default());

    // List all transactions
    let response = tracker.handle(&Request {
        tool: "list_transactions".into(),
        args: json!({}),
    }).await;

    Ok(())
}
```

### CLI Usage

The CLI is available as an example for testing:

```bash
# List categories
cargo run --example cli -- category list

# List transactions
cargo run --example cli -- transaction list

# Create a transaction
cargo run --example cli -- transaction create --amount 50.0 --kind shopping --desc "Groceries"

# Get a transaction by ID
cargo run --example cli -- transaction get --id 1

# Update a transaction
cargo run --example cli -- transaction update --id 1 --amount 60.0

# Delete a transaction
cargo run --example cli -- transaction delete --id 1

# Activity operations
cargo run --example cli -- activity create --start "09:00" --stop "10:00" --desc "Meeting"
cargo run --example cli -- activity list

# List places, tags, persons
cargo run --example cli -- place list
cargo run --example cli -- tag list
cargo run --example cli -- person list
```

### FFI (C API)

The library exposes a C-compatible FFI interface:

```c
#include "logbook.h"

// Get version
const char* version = tracker_version();

// One-shot JSON API (no handle management)
char* response = tracker_handle_json(
    "{\"tool\":\"list_transactions\",\"args\":{}}",
    NULL  // NULL for default database path
);
// Use response...
tracker_response_free(response);

// Or use handle-based API
void* handle = tracker_init(NULL);  // NULL for default database
char* response = tracker_request(handle, "{\"tool\":\"list_categories\",\"args\":{}}");
// Use response...
tracker_response_free(response);
tracker_free(handle);
```

#### Compile with FFI

**Windows (MSVC):**
```bash
cl test.c /Fe:test.exe /I include /link logbook.lib
```

**Linux/macOS (GCC/Clang):**
```bash
gcc test.c -o test -I include -L . -llogbook -Wl,-rpath,.
```

## Supported Tools/Operations

| Tool | Description |
|------|-------------|
| `create_transaction` | Create a new transaction |
| `get_transaction` | Get transaction by ID |
| `list_transactions` | List transactions (with optional filters) |
| `update_transaction` | Update a transaction |
| `delete_transaction` | Delete a transaction |
| `create_activity` | Create a new activity |
| `get_activity` | Get activity by ID |
| `list_activities` | List activities |
| `update_activity` | Update an activity |
| `delete_activity` | Delete an activity |
| `list_categories` | List all categories |
| `delete_category` | Delete a category |
| `list_places` | List all places |
| `delete_place` | Delete a place |
| `list_tags` | List all tags |
| `delete_tag` | Delete a tag |
| `list_persons` | List all persons |
| `delete_person` | Delete a person |

## Request/Response Format

### Request

```json
{
    "tool": "create_transaction",
    "args": {
        "amount": 50.0,
        "kind": "shopping",
        "description": "Groceries",
        "category": "Food",
        "place": "Supermarket",
        "tags": ["essential", "weekly"],
        "persons": ["John"]
    }
}
```

### Response

```json
{
    "success": true,
    "data": { "id": 1 },
    "message": "Transaction #1 created",
    "error": null
}
```

## Project Structure

```
logbook/
├── src/
│   ├── lib.rs          # Library entry point
│   ├── api.rs          # Request/Response handling
│   ├── error.rs        # Error types
│   ├── tracker/        # Tracker implementation
│   ├── types/          # Data types
│   ├── db/             # Database operations
│   └── ffi/            # C FFI bindings
├── examples/
│   ├── cli.rs          # CLI for testing
│   └── test_ffi.rs     # FFI test example
├── include/
│   └── logbook.h    # C header file
└── Cargo.toml
```

## Dependencies

- **sqlx**: Async SQLite database access
- **tokio**: Async runtime
- **serde/serde_json**: JSON serialization
- **chrono**: Date/time handling
- **thiserror**: Error handling
- **dirs**: Platform-specific directory paths

## Testing

```bash
# Run unit tests
cargo test

# Run CLI example
cargo run --example cli -- category list

# Run FFI test
cargo run --example test_ffi
```

## License

MIT
