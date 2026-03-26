/**
 * Test DLL loading and basic functionality
 * Compile: cl test_dll.c /Fe:test_dll.exe
 * Run: test_dll.exe
 */

#include <stdio.h>
#include <windows.h>

typedef void* (*logbook_init_fn)(const char* db_path);
typedef void (*logbook_free_fn)(void* handle);
typedef char* (*logbook_request_fn)(void* handle, const char* json_request);
typedef void (*logbook_response_free_fn)(char* response);
typedef char* (*logbook_handle_json_fn)(const char* json_request, const char* db_path);
typedef const char* (*logbook_version_fn)(void);

int main(void) {
    printf("Loading logbook.dll...\n");

    HMODULE dll = LoadLibraryA("logbook.dll");
    if (!dll) {
        printf("Failed to load DLL: Error %lu\n", GetLastError());
        return 1;
    }
    printf("DLL loaded successfully!\n\n");

    // Load functions
    logbook_version_fn version = (logbook_version_fn)GetProcAddress(dll, "logbook_version");
    logbook_init_fn init = (logbook_init_fn)GetProcAddress(dll, "logbook_init");
    logbook_free_fn free_handle = (logbook_free_fn)GetProcAddress(dll, "logbook_free");
    logbook_request_fn request = (logbook_request_fn)GetProcAddress(dll, "logbook_request");
    logbook_response_free_fn free_response = (logbook_response_free_fn)GetProcAddress(dll, "logbook_response_free");
    logbook_handle_json_fn handle_json = (logbook_handle_json_fn)GetProcAddress(dll, "logbook_handle_json");
    
    if (!version || !init || !free_handle || !request || !free_response || !handle_json) {
        printf("Failed to load functions: Error %lu\n", GetLastError());
        FreeLibrary(dll);
        return 1;
    }
    printf("All functions loaded!\n\n");
    
    // Test version
    printf("Version: %s\n\n", version());

    // Test one-shot function
    printf("Testing logbook_handle_json...\n");
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
        printf("Logbook initialized.\n");

        const char* create_req = "{\"tool\":\"create_transaction\",\"args\":{\"amount\":50.0,\"kind\":\"test\",\"description\":\"DLL test\"}}";
        printf("Creating transaction: %s\n", create_req);

        response = request(handle, create_req);
        if (response) {
            printf("Response: %s\n", response);
            free_response(response);
        }

        free_handle(handle);
        printf("Logbook freed.\n");
    } else {
        printf("Failed to initialize logbook.\n");
    }

    FreeLibrary(dll);
    printf("\nDLL unloaded. Test complete!\n");
    return 0;
}
