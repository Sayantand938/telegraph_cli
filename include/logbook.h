/**
 * logbook FFI Header
 *
 * C-style FFI for the logbook library.
 * All functions accept JSON strings and return JSON strings.
 *
 * Link against: logbook.dll (Windows), liblogbook.so (Linux), liblogbook.dylib (macOS)
 */

#ifndef LOGBOOK_FFI_H
#define LOGBOOK_FFI_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#ifdef _WIN32
    #ifdef LOGBOOK_LIB
        #define LOGBOOK_API __declspec(dllexport)
    #else
        #define LOGBOOK_API __declspec(dllimport)
    #endif
#else
    #define LOGBOOK_API __attribute__((visibility("default")))
#endif

/**
 * Opaque handle to a Logbook instance.
 */
typedef struct LogbookHandle LogbookHandle;

/**
 * Initialize the library and create a logbook instance.
 *
 * @param db_path Path to database file, or NULL for default location.
 * @return Pointer to LogbookHandle, or NULL on error.
 */
LOGBOOK_API LogbookHandle* logbook_init(const char* db_path);

/**
 * Free a logbook instance.
 *
 * @param handle Pointer to LogbookHandle (must be from logbook_init).
 */
LOGBOOK_API void logbook_free(LogbookHandle* handle);

/**
 * Send a request to the logbook and get a response.
 *
 * @param handle Pointer to LogbookHandle.
 * @param json_request JSON request string (null-terminated).
 * @return JSON response string (caller must free with logbook_response_free),
 *         or NULL on error.
 */
LOGBOOK_API char* logbook_request(LogbookHandle* handle, const char* json_request);

/**
 * Free a response string returned by logbook_request.
 *
 * @param response Pointer to response string (must be from logbook_request).
 */
LOGBOOK_API void logbook_response_free(char* response);

/**
 * Convenience function: JSON in, JSON out (no handle management needed).
 * Creates and destroys a logbook internally.
 *
 * @param json_request JSON request string (null-terminated).
 * @param db_path Path to database file, or NULL for default location.
 * @return JSON response string (caller must free with logbook_response_free),
 *         or NULL on error.
 */
LOGBOOK_API char* logbook_handle_json(const char* json_request, const char* db_path);

/**
 * Get the library version string.
 *
 * @return Static string with version (no need to free).
 */
LOGBOOK_API const char* logbook_version(void);

#ifdef __cplusplus
}
#endif

#endif /* TX_TRACKER_FFI_H */
