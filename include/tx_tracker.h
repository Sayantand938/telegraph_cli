/**
 * tx_tracker FFI Header
 * 
 * C-style FFI for the tx_tracker library.
 * All functions accept JSON strings and return JSON strings.
 * 
 * Link against: tx_tracker.dll (Windows), libtx_tracker.so (Linux), libtx_tracker.dylib (macOS)
 */

#ifndef TX_TRACKER_FFI_H
#define TX_TRACKER_FFI_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#ifdef _WIN32
    #ifdef TX_TRACKER_LIB
        #define TX_TRACKER_API __declspec(dllexport)
    #else
        #define TX_TRACKER_API __declspec(dllimport)
    #endif
#else
    #define TX_TRACKER_API __attribute__((visibility("default")))
#endif

/**
 * Opaque handle to a Tracker instance.
 */
typedef struct TrackerHandle TrackerHandle;

/**
 * Initialize the library and create a tracker instance.
 * 
 * @param db_path Path to database file, or NULL for default location.
 * @return Pointer to TrackerHandle, or NULL on error.
 */
TX_TRACKER_API TrackerHandle* tracker_init(const char* db_path);

/**
 * Free a tracker instance.
 * 
 * @param handle Pointer to TrackerHandle (must be from tracker_init).
 */
TX_TRACKER_API void tracker_free(TrackerHandle* handle);

/**
 * Send a request to the tracker and get a response.
 * 
 * @param handle Pointer to TrackerHandle.
 * @param json_request JSON request string (null-terminated).
 * @return JSON response string (caller must free with tracker_response_free),
 *         or NULL on error.
 */
TX_TRACKER_API char* tracker_request(TrackerHandle* handle, const char* json_request);

/**
 * Free a response string returned by tracker_request.
 * 
 * @param response Pointer to response string (must be from tracker_request).
 */
TX_TRACKER_API void tracker_response_free(char* response);

/**
 * Convenience function: JSON in, JSON out (no handle management needed).
 * Creates and destroys a tracker internally.
 * 
 * @param json_request JSON request string (null-terminated).
 * @param db_path Path to database file, or NULL for default location.
 * @return JSON response string (caller must free with tracker_response_free),
 *         or NULL on error.
 */
TX_TRACKER_API char* tracker_handle_json(const char* json_request, const char* db_path);

/**
 * Get the library version string.
 * 
 * @return Static string with version (no need to free).
 */
TX_TRACKER_API const char* tracker_version(void);

#ifdef __cplusplus
}
#endif

#endif /* TX_TRACKER_FFI_H */
