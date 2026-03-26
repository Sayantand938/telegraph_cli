# tx_tracker CLI Cheatsheet

Quick reference for all CLI commands. All commands use the format:
```bash
cargo run --example cli -- <domain> <action> [options]
```

---

## 📝 Transaction Commands

### Create Transaction
```bash
cargo run --example cli -- transaction create \
  --amount 50.0 \
  --kind shopping \
  --desc "Groceries" \
  [--category "Food"] \
  [--place "Supermarket"] \
  [--tags "urgent,weekly"] \
  [--persons "John,Jane"]
```

| Argument | Required | Description |
|----------|----------|-------------|
| `--amount` | ✅ | Transaction amount (e.g., `50.0`) |
| `--kind` | ✅ | Transaction type (e.g., `shopping`, `entertainment`) |
| `--desc` | ❌ | Description (default: empty) |
| `--category` | ❌ | Category name (auto-created if not exists) |
| `--place` | ❌ | Place name (auto-created if not exists) |
| `--tags` | ❌ | Comma-separated tags |
| `--persons` | ❌ | Comma-separated person names |

---

### Get Transaction
```bash
cargo run --example cli -- transaction get --id 1
```

| Argument | Required | Description |
|----------|----------|-------------|
| `--id` | ✅ | Transaction ID |

---

### List Transactions
```bash
cargo run --example cli -- transaction list \
  [--kind "shopping"]
```

| Argument | Required | Description |
|----------|----------|-------------|
| `--kind` | ❌ | Filter by kind |

---

### Update Transaction
```bash
cargo run --example cli -- transaction update \
  --id 1 \
  [--amount 60.0] \
  [--kind "entertainment"] \
  [--desc "Updated description"] \
  [--category "Food"] \
  [--place "Mall"]
```

| Argument | Required | Description |
|----------|----------|-------------|
| `--id` | ✅ | Transaction ID to update |
| `--amount` | ❌ | New amount |
| `--kind` | ❌ | New kind |
| `--desc` | ❌ | New description |
| `--category` | ❌ | New category name |
| `--place` | ❌ | New place name |

---

### Delete Transaction
```bash
cargo run --example cli -- transaction delete --id 1
```

| Argument | Required | Description |
|----------|----------|-------------|
| `--id` | ✅ | Transaction ID to delete |

---

## 📅 Activity Commands

### Create Activity
```bash
cargo run --example cli -- activity create \
  --start "09:00" \
  --stop "10:00" \
  --desc "Team Meeting" \
  [--category "Work"] \
  [--place "Office"] \
  [--tags "meeting,team"] \
  [--persons "Alice,Bob"]
```

| Argument | Required | Description |
|----------|----------|-------------|
| `--start` | ✅ | Start time (e.g., `09:00`) |
| `--stop` | ✅ | Stop time (e.g., `10:00`) |
| `--desc` | ❌ | Description (default: empty) |
| `--category` | ❌ | Category name |
| `--place` | ❌ | Place name |
| `--tags` | ❌ | Comma-separated tags |
| `--persons` | ❌ | Comma-separated person names |

---

### Get Activity
```bash
cargo run --example cli -- activity get --id 1
```

| Argument | Required | Description |
|----------|----------|-------------|
| `--id` | ✅ | Activity ID |

---

### List Activities
```bash
cargo run --example cli -- activity list
```
*(No arguments - lists all activities)*

---

### Update Activity
```bash
cargo run --example cli -- activity update \
  --id 1 \
  [--start "10:00"] \
  [--stop "11:00"] \
  [--desc "Updated meeting"] \
  [--category "Work"] \
  [--place "Conference Room"]
```

| Argument | Required | Description |
|----------|----------|-------------|
| `--id` | ✅ | Activity ID to update |
| `--start` | ❌ | New start time |
| `--stop` | ❌ | New stop time |
| `--desc` | ❌ | New description |
| `--category` | ❌ | New category name |
| `--place` | ❌ | New place name |

---

### Delete Activity
```bash
cargo run --example cli -- activity delete --id 1
```

| Argument | Required | Description |
|----------|----------|-------------|
| `--id` | ✅ | Activity ID to delete |

---

## ✅ Todo Commands

### Create Todo
```bash
cargo run --example cli -- todo create \
  --desc "Buy groceries" \
  [--status "pending"] \
  [--priority "high"] \
  [--due-date "2026-04-01"] \
  [--category "Shopping"] \
  [--place "Supermarket"] \
  [--tags "urgent,home"] \
  [--persons "John"]
```

| Argument | Required | Description |
|----------|----------|-------------|
| `--desc` | ✅ | Todo description |
| `--status` | ❌ | Status: `pending`, `in_progress`, `completed`, `cancelled` (default: `pending`) |
| `--priority` | ❌ | Priority: `low`, `medium`, `high` |
| `--due-date` | ❌ | Due date in ISO format (e.g., `2026-04-01`) |
| `--category` | ❌ | Category name |
| `--place` | ❌ | Place name |
| `--tags` | ❌ | Comma-separated tags |
| `--persons` | ❌ | Comma-separated person names |

---

### Get Todo
```bash
cargo run --example cli -- todo get --id 1
```

| Argument | Required | Description |
|----------|----------|-------------|
| `--id` | ✅ | Todo ID |

---

### List Todos
```bash
cargo run --example cli -- todo list \
  [--status "pending"] \
  [--priority "high"]
```

| Argument | Required | Description |
|----------|----------|-------------|
| `--status` | ❌ | Filter by status |
| `--priority` | ❌ | Filter by priority |

---

### Update Todo
```bash
cargo run --example cli -- todo update \
  --id 1 \
  [--desc "Updated description"] \
  [--status "in_progress"] \
  [--priority "medium"] \
  [--due-date "2026-04-05"] \
  [--category "Work"] \
  [--place "Office"]
```

| Argument | Required | Description |
|----------|----------|-------------|
| `--id` | ✅ | Todo ID to update |
| `--desc` | ❌ | New description |
| `--status` | ❌ | New status |
| `--priority` | ❌ | New priority |
| `--due-date` | ❌ | New due date |
| `--category` | ❌ | New category name |
| `--place` | ❌ | New place name |

---

### Complete Todo
```bash
cargo run --example cli -- todo complete --id 1
```

| Argument | Required | Description |
|----------|----------|-------------|
| `--id` | ✅ | Todo ID to complete |

*Sets status to `completed` and records completion timestamp*

---

### Delete Todo
```bash
cargo run --example cli -- todo delete --id 1
```

| Argument | Required | Description |
|----------|----------|-------------|
| `--id` | ✅ | Todo ID to delete |

---

## 📂 Category Commands

### List Categories
```bash
cargo run --example cli -- category list
```

---

### Delete Category
```bash
cargo run --example cli -- category delete --id 1
```

| Argument | Required | Description |
|----------|----------|-------------|
| `--id` | ✅ | Category ID to delete |

---

## 📍 Place Commands

### List Places
```bash
cargo run --example cli -- place list
```

---

### Delete Place
```bash
cargo run --example cli -- place delete --id 1
```

| Argument | Required | Description |
|----------|----------|-------------|
| `--id` | ✅ | Place ID to delete |

---

## 🏷️ Tag Commands

### List Tags
```bash
cargo run --example cli -- tag list
```

---

### Delete Tag
```bash
cargo run --example cli -- tag delete --id 1
```

| Argument | Required | Description |
|----------|----------|-------------|
| `--id` | ✅ | Tag ID to delete |

---

## 👤 Person Commands

### List Persons
```bash
cargo run --example cli -- person list
```

---

### Delete Person
```bash
cargo run --example cli -- person delete --id 1
```

| Argument | Required | Description |
|----------|----------|-------------|
| `--id` | ✅ | Person ID to delete |

---

## 🔧 Common Patterns

### Multiple Tags/Persons
Use comma-separated values (no spaces):
```bash
--tags "urgent,work,deadline"
--persons "Alice,Bob,Charlie"
```

### Category/Place Auto-Creation
Categories and places are **auto-created** if they don't exist:
```bash
# Creates "Food" category if it doesn't exist
cargo run --example cli -- transaction create \
  --amount 50 --kind shopping --category "Food"
```

### Status Values
- **Todo Status**: `pending`, `in_progress`, `completed`, `cancelled`
- **Priority**: `low`, `medium`, `high`

---

## 📋 Quick Examples

```bash
# Create a high-priority todo due tomorrow
cargo run --example cli -- todo create \
  --desc "Finish project report" \
  --priority high \
  --due-date 2026-03-27 \
  --category Work

# List all pending high-priority todos
cargo run --example cli -- todo list \
  --status pending \
  --priority high

# Mark todo as in progress
cargo run --example cli -- todo update \
  --id 1 \
  --status in_progress

# Complete a todo
cargo run --example cli -- todo complete --id 1

# Create a shopping transaction
cargo run --example cli -- transaction create \
  --amount 120.50 \
  --kind shopping \
  --desc "Weekly groceries" \
  --category Food \
  --place "Supermarket"

# Log a 2-hour meeting
cargo run --example cli -- activity create \
  --start "14:00" \
  --stop "16:00" \
  --desc "Project planning" \
  --category Work
```
