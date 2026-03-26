# Flutter / Mobile Integration Guide

## Overview

The logbook library now provides a **unified command interface** that makes it trivial to integrate with Flutter, iOS, Android, or any other platform via FFI.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Flutter / Dart                            │
│                                                              │
│   String result = await logbook.command(                    │
│     "transaction create --amount 50 --kind shopping"        │
│   );                                                         │
└─────────────────────────────────────────────────────────────┘
                              ↓ calls via FFI
┌─────────────────────────────────────────────────────────────┐
│              logbook_command() (C FFI)                       │
│                                                              │
│   Parses command → Executes → Returns JSON                  │
└─────────────────────────────────────────────────────────────┘
                              ↓ uses
┌─────────────────────────────────────────────────────────────┐
│                    Rust Library                              │
│   Tracker → Domain Processors → DB → SQLite                 │
└─────────────────────────────────────────────────────────────┘
```

---

## FFI Function

### `logbook_command`

```c
// C header (include/logbook.h)
LOGBOOK_API char* logbook_command(
    const char* command,    // Command string
    const char* db_path     // Database path (NULL for default)
);

// Don't forget to free the response!
LOGBOOK_API void logbook_response_free(char* response);
```

---

## Command Format

```
<domain> <action> [--arg value] [--arg value] ...
```

### Domains

| Domain | Actions |
|--------|---------|
| `transaction` | create, get, list, update, delete |
| `activity` | create, get, list, update, delete |
| `todo` | create, get, list, update, delete, complete |
| `journal` | create, get, list, search, update, delete |
| `category` | list, delete |
| `place` | list, delete |
| `tag` | list, delete |
| `person` | list, delete |

---

## Usage Examples

### 1. Create Transaction

**Command:**
```
transaction create --amount 50.0 --kind shopping --description "Groceries" --category Food --place "Supermarket" --tags essential,weekly --persons Alice,Bob
```

**Dart/Flutter:**
```dart
import 'dart:ffi';
import 'dart:convert';

Future<Map<String, dynamic>> createTransaction() async {
  final command = "transaction create "
      "--amount 50.0 "
      "--kind shopping "
      "--description 'Groceries' "
      "--category Food "
      "--place 'Supermarket' "
      "--tags essential,weekly "
      "--persons Alice,Bob";
  
  final responsePtr = logbook_command(command.toNativeUtf8(), nullptr);
  final response = responsePtr.cast<Utf8>().toDartString();
  logbook_response_free(responsePtr);
  
  return jsonDecode(response);
}

// Response:
// {
//   "success": true,
//   "message": "Transaction #123 created",
//   "data": {"id": 123},
//   "error": null
// }
```

---

### 2. List Todos with Filters

**Command:**
```
todo list --status pending --priority high
```

**Dart/Flutter:**
```dart
Future<List<Todo>> getHighPriorityTodos() async {
  final command = "todo list --status pending --priority high";
  
  final responsePtr = logbook_command(command.toNativeUtf8(), nullptr);
  final response = jsonDecode(responsePtr.cast<Utf8>().toDartString());
  logbook_response_free(responsePtr);
  
  if (response['success']) {
    return (response['data'] as List)
        .map((json) => Todo.fromJson(json))
        .toList();
  }
  throw Exception(response['error']);
}
```

---

### 3. Search Journal (Full-Text Search)

**Command:**
```
journal search --query "meeting notes" --from 2026-03-01 --to 2026-03-31
```

**Dart/Flutter:**
```dart
Future<List<JournalEntry>> searchJournal(String query) async {
  final command = "journal search --query '$query'";
  
  final responsePtr = logbook_command(command.toNativeUtf8(), nullptr);
  final response = jsonDecode(responsePtr.cast<Utf8>().toDartString());
  logbook_response_free(responsePtr);
  
  if (response['success']) {
    return (response['data'] as List)
        .map((json) => JournalEntry.fromJson(json))
        .toList();
  }
  throw Exception(response['error']);
}
```

---

### 4. List Reference Data

**Command:**
```
category list
place list
tag list
person list
```

**Dart/Flutter:**
```dart
Future<List<Category>> getCategories() async {
  final responsePtr = logbook_command(
    "category list".toNativeUtf8(), 
    nullptr
  );
  final response = jsonDecode(responsePtr.cast<Utf8>().toDartString());
  logbook_response_free(responsePtr);
  
  return (response['data'] as List)
      .map((json) => Category.fromJson(json))
      .toList();
}
```

---

### 5. Custom Database Path

```dart
// Use a specific database file
final dbPath = "/path/to/my.db".toNativeUtf8();
final responsePtr = logbook_command(
  "todo list".toNativeUtf8(),
  dbPath.cast()
);
```

---

## Response Format

All commands return JSON with this structure:

```json
{
  "success": true,
  "message": "Operation completed successfully",
  "data": { ... },      // Result data (if any)
  "error": null         // Error message (if failed)
}
```

### Success Response
```json
{
  "success": true,
  "message": "Transaction #123 created",
  "data": {"id": 123},
  "error": null
}
```

### Error Response
```json
{
  "success": false,
  "message": null,
  "data": null,
  "error": "Transaction not found"
}
```

---

## Key Benefits

### ✅ No JSON Construction Needed
Just pass simple command strings!

**Before (JSON approach):**
```dart
final request = jsonEncode({
  "tool": "create_transaction",
  "args": {
    "amount": 50.0,
    "kind": "shopping",
    "description": "Groceries",
    // ...
  }
});
```

**After (Command approach):**
```dart
final command = "transaction create --amount 50.0 --kind shopping --description 'Groceries'";
```

### ✅ Auto-Resolution of Names
Categories, places, tags, and persons are auto-created if they don't exist!

```
transaction create --category "Food" --place "Supermarket"
// "Food" category and "Supermarket" place auto-created if needed
```

### ✅ Consistent Interface
Same command format works for:
- CLI: `cargo run -- transaction create ...`
- FFI: `logbook_command("transaction create ...", NULL)`
- Flutter: Same as FFI via dart:ffi

---

## Complete Flutter Example

```dart
import 'dart:ffi';
import 'dart:convert';
import 'package:ffi/ffi.dart';

class LogbookDB {
  final DynamicLibrary _lib;
  late final LogbookCommand _command;
  late final LogbookResponseFree _free;

  LogbookDB._(this._lib) {
    _command = _lib
        .lookup<NativeFunction<logbook_command_native>>('logbook_command')
        .asFunction();
    _free = _lib
        .lookup<NativeFunction<logbook_response_free_native>>('logbook_response_free')
        .asFunction();
  }

  static Future<LogbookDB> open({String? dbPath}) async {
    final lib = DynamicLibrary.open('logbook.dll'); // or .so / .dylib
    return LogbookDB._(lib);
  }

  Future<Map<String, dynamic>> execute(String command) async {
    final commandPtr = command.toNativeUtf8();
    final responsePtr = _command(commandPtr.cast(), nullptr);
    
    if (responsePtr == nullptr) {
      throw Exception("FFI call failed");
    }
    
    final responseStr = responsePtr.cast<Utf8>().toDartString();
    _free(responsePtr);
    calloc.free(commandPtr);
    
    return jsonDecode(responseStr);
  }

  // Convenience methods
  Future<int> createTransaction({
    required double amount,
    required String kind,
    String? description,
    String? category,
    String? place,
    List<String>? tags,
    List<String>? persons,
  }) async {
    var cmd = "transaction create "
        "--amount $amount "
        "--kind $kind "
        "--description '$description'";
    
    if (category != null) cmd += " --category '$category'";
    if (place != null) cmd += " --place '$place'";
    if (tags != null) cmd += " --tags ${tags.join(',')}";
    if (persons != null) cmd += " --persons ${persons.join(',')}";
    
    final result = await execute(cmd);
    if (!result['success']) throw Exception(result['error']);
    return result['data']['id'] as int;
  }

  Future<List<Map<String, dynamic>>> listTodos({
    String? status,
    String? priority,
  }) async {
    var cmd = "todo list";
    if (status != null) cmd += " --status $status";
    if (priority != null) cmd += " --priority $priority";
    
    final result = await execute(cmd);
    if (!result['success']) throw Exception(result['error']);
    return List<Map<String, dynamic>>.from(result['data']);
  }
}

// Type definitions
typedef logbook_command_native = Pointer<Int8> Function(
  Pointer<Int8> command,
  Pointer<Int8> dbPath,
);
typedef LogbookCommand = Pointer<Int8> Function(
  Pointer<Int8> command,
  Pointer<Int8> dbPath,
);

typedef logbook_response_free_native = Void Function(Pointer<Int8>);
typedef LogbookResponseFree = void Function(Pointer<Int8>);
```

---

## Migration from JSON API

If you were using the JSON-based `logbook_handle_json`:

**Old (JSON):**
```dart
final json = jsonEncode({
  "tool": "list_transactions",
  "args": {"kind": "shopping"}
});
final result = logbook_handle_json(json, nullptr);
```

**New (Command):**
```dart
final result = logbook_command("transaction list --kind shopping", nullptr);
```

**Much simpler!** 🎉

---

## Testing

Test commands directly in your terminal:

```bash
# Test via CLI
cargo run --example cli -- transaction create --amount 50 --kind shopping

# Test via command interface
cargo test command
```

---

## Summary

The command interface provides:
- ✅ Simple string-based API (no JSON construction)
- ✅ Auto-resolution of names (categories, places, tags, persons)
- ✅ Consistent format across CLI, FFI, Flutter
- ✅ Full type safety via JSON responses
- ✅ Zero boilerplate in Flutter/Dart

**Start integrating in minutes, not hours!** 🚀
