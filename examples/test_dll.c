/**
 * Test DLL loading and basic functionality
 * Compile: cl test_dll.c /Fe:test_dll.exe
 * Run: test_dll.exe
 */

#include <stdio.h>
#include <windows.h>

typedef void* (*tracker_init_fn)(const char* db_path);
typedef void (*tracker_free_fn)(void* handle);
typedef char* (*tracker_request_fn)(void* handle, const char* json_request);
typedef void (*tracker_response_free_fn)(char* response);
typedef char* (*tracker_handle_json_fn)(const char* json_request, const char* db_path);
typedef const char* (*tracker_version_fn)(void);

int main(void) {
    printf("Loading tx_tracker.dll...\n");
    
    HMODULE dll = LoadLibraryA("tx_tracker.dll");
    if (!dll) {
        printf("Failed to load DLL: Error %lu\n", GetLastError());
        return 1;
    }
    printf("DLL loaded successfully!\n\n");
    
    // Load functions
    tracker_version_fn version = (tracker_version_fn)GetProcAddress(dll, "tracker_version");
    tracker_init_fn init = (tracker_init_fn)GetProcAddress(dll, "tracker_init");
    tracker_free_fn free_handle = (tracker_free_fn)GetProcAddress(dll, "tracker_free");
    tracker_request_fn request = (tracker_request_fn)GetProcAddress(dll, "tracker_request");
    tracker_response_free_fn free_response = (tracker_response_free_fn)GetProcAddress(dll, "tracker_response_free");
    tracker_handle_json_fn handle_json = (tracker_handle_json_fn)GetProcAddress(dll, "tracker_handle_json");
    
    if (!version || !init || !free_handle || !request || !free_response || !handle_json) {
        printf("Failed to load functions: Error %lu\n", GetLastError());
        FreeLibrary(dll);
        return 1;
    }
    printf("All functions loaded!\n\n");
    
    // Test version
    printf("Version: %s\n\n", version());
    
    // Test one-shot function
    printf("Testing tracker_handle_json...\n");
    const char* json_req = "{\"tool\":\"list_transactions\",\"args\":null}";
    printf("Request: %s\n", json_req);
    
    char* response = handle_json(json_req, NULL);
    if (response) {
        printf("Response: %s\n", response);
        free_response(response);
        printf("\nSuccess! DLL is working.\n");
    } else {
        printf("Request failed!\n");
    }
    
    // Test handle-based API
    printf("\nTesting handle-based API...\n");
    void* handle = init(NULL);
    if (handle) {
        printf("Tracker initialized.\n");
        
        const char* create_req = "{\"tool\":\"create_transaction\",\"args\":{\"amount\":50.0,\"kind\":\"test\",\"description\":\"DLL test\"}}";
        printf("Creating transaction: %s\n", create_req);
        
        response = request(handle, create_req);
        if (response) {
            printf("Response: %s\n", response);
            free_response(response);
        }
        
        free_handle(handle);
        printf("Tracker freed.\n");
    } else {
        printf("Failed to initialize tracker.\n");
    }
    
    FreeLibrary(dll);
    printf("\nDLL unloaded. Test complete!\n");
    return 0;
}
