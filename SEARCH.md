# Unified Search Command

## Overview

The unified search command allows you to search across all domains with flexible filters, sorting, and pagination.

---

## Command Format

```
search <domain> [filters...] [--limit N] [--offset N] [--order-by column] [--order ASC|DESC]
```

---

## Usage Examples

### 1. Search Transactions

**Find all shopping transactions:**
```bash
search transaction --kind shopping
```

**Find transactions over $50:**
```bash
search transaction --amount 50
```

**Find transactions with limit:**
```bash
search transaction --kind shopping --limit 20
```

**Find transactions ordered by amount:**
```bash
search transaction --kind shopping --order-by amount --order DESC --limit 10
```

### 2. Search Todos

**Find pending todos:**
```bash
search todo --status pending
```

**Find high priority todos:**
```bash
search todo --priority high
```

**Find todos due soon:**
```bash
search todo --status pending --order-by due_date --limit 10
```

### 3. Search Journal

**Search by date:**
```bash
search journal --date 2026-03-26
```

**Find recent entries:**
```bash
search journal --order-by created_at --order DESC --limit 5
```

### 4. Search Categories/Places/Tags/Persons

**Find category by name:**
```bash
search category --name Food
```

**Find all places:**
```bash
search place --limit 100
```

---

## From Flutter/Dart

```dart
// Search transactions
final result = await logbook.execute(
  "search transaction --kind shopping --limit 20"
);

// Search pending todos
final todos = await logbook.execute(
  "search todo --status pending --order-by due_date"
);

// Search with pagination
final page1 = await logbook.execute(
  "search transaction --limit 20 --offset 0"
);
final page2 = await logbook.execute(
  "search transaction --limit 20 --offset 20"
);
```

---

## Supported Filters by Domain

### Transaction
- `kind` - Transaction type (shopping, entertainment, etc.)
- `amount` - Amount value
- `description` - Description text
- `category` / `category_id` - Category filter
- `place` / `place_id` - Place filter

### Activity
- `description` - Activity description
- `start_time` / `stop_time` - Time filters
- `category` / `category_id` - Category filter
- `place` / `place_id` - Place filter

### Todo
- `status` - pending, in_progress, completed
- `priority` - high, medium, low
- `description` - Todo description
- `due_date` - Due date filter
- `category` / `category_id` - Category filter
- `place` / `place_id` - Place filter

### Journal
- `date` - Entry date
- `content` - Content text (use search_journals for full-text)
- `category` / `category_id` - Category filter
- `place` / `place_id` - Place filter

### Reference Data (Categories, Places, Tags, Persons)
- `name` - Name filter

---

## Query Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `--limit N` | Maximum results to return | 100 |
| `--offset N` | Number of results to skip | 0 |
| `--order-by column` | Column to sort by | None |
| `--order ASC\|DESC` | Sort direction | ASC |

---

## Response Format

```json
{
  "success": true,
  "message": "5 result(s) found",
  "data": [
    {
      "id": 1,
      "amount": 50.0,
      "kind": "shopping",
      "description": "Groceries",
      ...
    },
    ...
  ],
  "error": null
}
```

---

## Advanced: Using WHERE Clause

For complex searches, use the `--where` parameter:

```bash
# Multiple conditions
search transaction --where "kind=shopping AND amount>50"

# With ordering
search todo --where "status=pending AND priority=high" --order-by due_date
```

**Operators supported:**
- `=` - Equals
- `!=` - Not equals
- `>` - Greater than
- `>=` - Greater than or equal
- `<` - Less than
- `<=` - Less than or equal

---

## Limitations

1. **Max limit**: 1000 results per query
2. **No JOINs**: Can only filter on the domain's own columns
3. **No full-text search**: For journal content, use `journal search --query "text"` instead
4. **Column whitelist**: Only predefined columns can be filtered

---

## Examples from Different Interfaces

### CLI (via command interface)
```bash
cargo run --example cli -- search transaction --kind shopping --limit 10
```

### FFI (C/Flutter)
```c
const char* result = logbook_command(
    "search transaction --kind shopping --limit 20",
    NULL
);
```

### Rust (direct API)
```rust
let result = execute_command(
    "search todo --status pending --limit 10",
    &tracker
).await?;
```

### JSON API
```json
{
  "tool": "search_transaction",
  "args": {
    "kind": "shopping",
    "limit": 20
  }
}
```

---

## Migration from List Commands

| Old Command | New Search Command |
|-------------|-------------------|
| `list_transactions --kind shopping` | `search transaction --kind shopping` |
| `list_todos --status pending` | `search todo --status pending` |
| `list_activities` | `search activity --limit 100` |

**Note**: List commands still work! Search is an alternative with more features.
