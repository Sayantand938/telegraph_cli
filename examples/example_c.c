/**
 * Example C program using logbook FFI
 *
 * Compile (Windows with MSVC):
 *   cl example_c.c logbook.lib
 *
 * Compile (Linux/macOS with gcc):
 *   gcc example_c.c -L. -llogbook -Wl,-rpath,.
 *
 * Run:
 *   ./example_c   (Linux/macOS)
 *   example_c.exe (Windows)
 */

#include <stdio.h>
#include <stdlib.h>
#include "logbook.h"

int main(void) {
    printf("logbook FFI Example\n");
    printf("===================\n\n");

    // Get version
    printf("Version: %s\n\n", logbook_version());

    // Method 1: Using handle (recommended for multiple requests)
    printf("Method 1: Using persistent handle\n");
    printf("----------------------------------\n");

    LogbookHandle* handle = logbook_init(NULL);
    if (handle == NULL) {
        fprintf(stderr, "Failed to initialize logbook\n");
        return 1;
    }

    // Create a transaction
    const char* create_req = "{\"tool\":\"create_transaction\",\"args\":{\"amount\":75.50,\"kind\":\"transport\",\"description\":\"Taxi\"}}";
    printf("Request: %s\n", create_req);

    char* response = logbook_request(handle, create_req);
    if (response) {
        printf("Response: %s\n\n", response);
        logbook_response_free(response);
    }

    // List transactions
    const char* list_req = "{\"tool\":\"list_transactions\",\"args\":null}";
    printf("Request: %s\n", list_req);

    response = logbook_request(handle, list_req);
    if (response) {
        printf("Response: %s\n\n", response);
        logbook_response_free(response);
    }

    logbook_free(handle);

    // Method 2: One-shot function (convenient for single requests)
    printf("Method 2: One-shot function\n");
    printf("---------------------------\n");

    const char* activity_req = "{\"tool\":\"list_activities\",\"args\":null}";
    printf("Request: %s\n", activity_req);

    response = logbook_handle_json(activity_req, NULL);
    if (response) {
        printf("Response: %s\n\n", response);
        logbook_response_free(response);
    }

    printf("Done!\n");
    return 0;
}
