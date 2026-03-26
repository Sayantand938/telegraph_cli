# logbook CLI Cheatsheet

Quick reference for all CLI commands. All commands use the format:
```bash
cargo run --example cli -- <domain> <action> [options]
```

---

## 🔍 Unified Search Commands

### Search Transactions
```bash
cargo run --example cli -- search transaction \
  --kind shopping \
  --limit 20 \
  [--order-by amount] \
  [--order DESC]
```

| Argument | Required | Description |
|----------|----------|-------------|
| `--kind` | ❌ | Filter by kind (e.g., `shopping`, `entertainment`) |
| `--amount` | ❌ | Filter by amount |
| `--category` | ❌ | Filter by category name |
| `--place` | ❌ | Filter by place name |
| `--limit` | ❌ | Max results (default: 100, max: 1000) |
| `--offset` | ❌ | Skip N results (for pagination) |
| `--order-by` | ❌ | Column to sort by |
| `--order` | ❌ | Sort direction: `ASC` or `DESC` |

---

### Search Todos
```bash
cargo run --example cli -- search todo \
  --status pending \
  --priority high \
  --limit 10 \
  [--order-by due_date]
```

| Argument | Required | Description |
|----------|----------|-------------|
| `--status` | ❌ | Filter by status (`pending`, `in_progress`, `completed`) |
| `--priority` | ❌ | Filter by priority (`high`, `medium`, `low`) |
| `--description` | ❌ | Filter by description |
| `--due-date` | ❌ | Filter by due date |
| `--limit` | ❌ | Max results |
| `--order-by` | ❌ | Column to sort by |

---

### Search Activities
```bash
cargo run --example cli -- search activity \
  --description "Meeting" \
  --limit 20
```

| Argument | Required | Description |
|----------|----------|-------------|
| `--description` | ❌ | Filter by description |
| `--start-time` | ❌ | Filter by start time |
| `--category` | ❌ | Filter by category |
| `--limit` | ❌ | Max results |

---

### Search Journal
```bash
cargo run --example cli -- search journal \
  --date 2026-03-26 \
  --limit 10
```

| Argument | Required | Description |
|----------|----------|-------------|
| `--date` | ❌ | Filter by date |
| `--category` | ❌ | Filter by category |
| `--limit` | ❌ | Max results |

---

### Search Reference Data
```bash
# Search categories
cargo run --example cli -- search category --name Food

# Search places
cargo run --example cli -- search place --name "Supermarket"

# Search tags
cargo run --example cli -- search tag --name urgent

# Search persons
cargo run --example cli -- search person --name "John"
```

---

### Advanced Search Examples

```bash
# Find expensive shopping transactions
cargo run --example cli -- search transaction \
  --kind shopping \
  --order-by amount \
  --order DESC \
  --limit 5

# Find pending high-priority todos
cargo run --example cli -- search todo \
  --status pending \
  --priority high \
  --order-by due_date \
  --limit 10

# Paginated search (page 2)
cargo run --example cli -- search transaction \
  --limit 20 \
  --offset 20
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
