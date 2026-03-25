"""
Example Python script using tx_tracker FFI via ctypes

Requirements:
    - tx_tracker.dll (Windows) or libtx_tracker.so (Linux) or libtx_tracker.dylib (macOS)
    - Place the library in the same directory or in system library path

Run:
    python example_python.py
"""

import ctypes
import json
from ctypes import c_char_p, c_void_p, POINTER

# Load the library
import os
import sys

if sys.platform == "win32":
    lib = ctypes.CDLL("./tx_tracker.dll")
elif sys.platform == "darwin":
    lib = ctypes.CDLL("./libtx_tracker.dylib")
else:
    lib = ctypes.CDLL("./libtx_tracker.so")

# Define function signatures
lib.tracker_init.argtypes = [c_char_p]
lib.tracker_init.restype = c_void_p

lib.tracker_free.argtypes = [c_void_p]
lib.tracker_free.restype = None

lib.tracker_request.argtypes = [c_void_p, c_char_p]
lib.tracker_request.restype = c_char_p

lib.tracker_response_free.argtypes = [c_char_p]
lib.tracker_response_free.restype = None

lib.tracker_handle_json.argtypes = [c_char_p, c_char_p]
lib.tracker_response_free.restype = c_char_p

lib.tracker_version.argtypes = []
lib.tracker_version.restype = c_char_p


def main():
    print("tx_tracker FFI Python Example")
    print("=============================\n")

    # Get version
    version = lib.tracker_version().decode("utf-8")
    print(f"Version: {version}\n")

    # Method 1: Using handle (recommended for multiple requests)
    print("Method 1: Using persistent handle")
    print("----------------------------------")

    handle = lib.tracker_init(None)
    if not handle:
        print("Failed to initialize tracker")
        return 1

    # Create a transaction
    create_req = json.dumps({
        "tool": "create_transaction",
        "args": {"amount": 100.0, "kind": "shopping", "description": "Groceries"}
    })
    print(f"Request: {create_req}")

    response = lib.tracker_request(handle, create_req.encode("utf-8"))
    if response:
        result = json.loads(response.decode("utf-8"))
        print(f"Response: {json.dumps(result, indent=2)}\n")
        lib.tracker_response_free(response)

    # List transactions
    list_req = json.dumps({"tool": "list_transactions", "args": None})
    print(f"Request: {list_req}")

    response = lib.tracker_request(handle, list_req.encode("utf-8"))
    if response:
        result = json.loads(response.decode("utf-8"))
        print(f"Response: {json.dumps(result, indent=2)}\n")
        lib.tracker_response_free(response)

    lib.tracker_free(handle)

    # Method 2: One-shot function
    print("Method 2: One-shot function")
    print("---------------------------")

    activity_req = json.dumps({"tool": "list_activities", "args": None})
    print(f"Request: {activity_req}")

    response = lib.tracker_handle_json(activity_req.encode("utf-8"), None)
    if response:
        result = json.loads(response.decode("utf-8"))
        print(f"Response: {json.dumps(result, indent=2)}\n")
        lib.tracker_response_free(response)

    print("Done!")
    return 0


if __name__ == "__main__":
    exit(main())
