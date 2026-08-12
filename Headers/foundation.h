/*
 * TontooFoundation - C Header
 * TontooOS Foundation Framework
 *
 * This header provides C bindings for the Foundation library.
 */

#ifndef TONTOO_FOUNDATION_H
#define TONTOO_FOUNDATION_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ======================== */
/* URL                      */
/* ======================== */

/**
 * Parse a URL string and return a handle.
 *
 * @param url The URL string to parse
 * @return A handle (>= 0) on success, negative on error
 */
int foundation_url_parse(const char *url);

/**
 * Get the scheme of a parsed URL.
 *
 * @param handle The URL handle from foundation_url_parse
 * @return The scheme string (do NOT free), or NULL if not present
 */
const char *foundation_url_scheme(int handle);

/**
 * Get the host of a parsed URL.
 *
 * @param handle The URL handle
 * @return The host string (do NOT free), or NULL if not present
 */
const char *foundation_url_host(int handle);

/**
 * Get the port of a parsed URL.
 *
 * @param handle The URL handle
 * @return The port number, or -1 if not present
 */
int foundation_url_port(int handle);

/**
 * Get the path of a parsed URL.
 *
 * @param handle The URL handle
 * @return The path string (do NOT free)
 */
const char *foundation_url_path(int handle);

/**
 * Get the full absolute string of a parsed URL.
 *
 * @param handle The URL handle
 * @return The full URL string (must be freed with foundation_free_string)
 */
char *foundation_url_absolute_string(int handle);

/**
 * Free a parsed URL handle.
 *
 * @param handle The URL handle to free
 */
void foundation_url_free(int handle);

/* ======================== */
/* Date                     */
/* ======================== */

/**
 * Get the current Unix timestamp in seconds.
 *
 * @return The current timestamp
 */
int64_t foundation_date_now(void);

/**
 * Get the current Unix timestamp in milliseconds.
 *
 * @return The current timestamp in milliseconds
 */
int64_t foundation_date_now_millis(void);

/**
 * Format a timestamp as ISO 8601 string.
 *
 * @param timestamp The Unix timestamp in seconds
 * @return The formatted string (must be freed with foundation_free_string)
 */
char *foundation_date_format_iso8601(int64_t timestamp);

/**
 * Format a timestamp using a custom format string.
 *
 * @param timestamp The Unix timestamp in seconds
 * @param format The format string (e.g., "%Y-%m-%d %H:%M:%S")
 * @return The formatted string (must be freed with foundation_free_string)
 */
char *foundation_date_format(int64_t timestamp, const char *format);

/**
 * Add seconds to a timestamp.
 *
 * @param timestamp The base timestamp
 * @param seconds The seconds to add
 * @return The new timestamp
 */
int64_t foundation_date_add_seconds(int64_t timestamp, int64_t seconds);

/**
 * Add days to a timestamp.
 *
 * @param timestamp The base timestamp
 * @param days The days to add
 * @return The new timestamp
 */
int64_t foundation_date_add_days(int64_t timestamp, int64_t days);

/* ======================== */
/* JSON                     */
/* ======================== */

/**
 * Check if a string is valid JSON.
 *
 * @param json The JSON string to validate
 * @return 1 if valid, 0 if invalid
 */
int foundation_json_is_valid(const char *json);

/**
 * Minify a JSON string (remove whitespace).
 *
 * @param json The JSON string to minify
 * @return The minified string (must be freed with foundation_free_string)
 */
char *foundation_json_minify(const char *json);

/**
 * Pretty-print a JSON string.
 *
 * @param json The JSON string to pretty-print
 * @return The formatted string (must be freed with foundation_free_string)
 */
char *foundation_json_pretty(const char *json);

/* ======================== */
/* UserDefaults             */
/* ======================== */

/**
 * Set a string value in UserDefaults.
 *
 * @param key The key
 * @param value The value
 * @return 0 on success, negative on error
 */
int foundation_defaults_set_string(const char *key, const char *value);

/**
 * Get a string value from UserDefaults.
 *
 * @param key The key
 * @return The value string (must be freed with foundation_free_string), or NULL if not found
 */
char *foundation_defaults_get_string(const char *key);

/**
 * Set an integer value in UserDefaults.
 *
 * @param key The key
 * @param value The value
 * @return 0 on success, negative on error
 */
int foundation_defaults_set_int(const char *key, int64_t value);

/**
 * Get an integer value from UserDefaults.
 *
 * @param key The key
 * @return The value, or 0 if not found
 */
int64_t foundation_defaults_get_int(const char *key);

/**
 * Set a boolean value in UserDefaults.
 *
 * @param key The key
 * @param value The value (0 or 1)
 * @return 0 on success, negative on error
 */
int foundation_defaults_set_bool(const char *key, int value);

/**
 * Get a boolean value from UserDefaults.
 *
 * @param key The key
 * @return 1 if true, 0 if false or not found
 */
int foundation_defaults_get_bool(const char *key);

/**
 * Remove a key from UserDefaults.
 *
 * @param key The key
 * @return 0 on success, negative on error
 */
int foundation_defaults_remove(const char *key);

/**
 * Save UserDefaults to disk.
 *
 * @return 0 on success, negative on error
 */
int foundation_defaults_save(void);

/* ======================== */
/* Process                  */
/* ======================== */

/**
 * Get the current process ID.
 *
 * @return The process ID
 */
uint32_t foundation_process_id(void);

/**
 * Get the process name.
 *
 * @return The process name string (must be freed with foundation_free_string)
 */
char *foundation_process_name(void);

/**
 * Get the OS version string.
 *
 * @return The version string (do NOT free)
 */
const char *foundation_os_version_string(void);

/**
 * Get the physical memory in bytes.
 *
 * @return The memory size in bytes
 */
uint64_t foundation_physical_memory(void);

/**
 * Get the number of processors.
 *
 * @return The processor count
 */
uint32_t foundation_processor_count(void);

/* ======================== */
/* String Utilities         */
/* ======================== */

/**
 * Check if a string contains a substring.
 *
 * @param haystack The string to search in
 * @param needle The substring to find
 * @return 1 if found, 0 if not
 */
int foundation_string_contains(const char *haystack, const char *needle);

/**
 * Check if a string starts with a prefix.
 *
 * @param s The string
 * @param prefix The prefix
 * @return 1 if starts with, 0 if not
 */
int foundation_string_has_prefix(const char *s, const char *prefix);

/**
 * Check if a string ends with a suffix.
 *
 * @param s The string
 * @param suffix The suffix
 * @return 1 if ends with, 0 if not
 */
int foundation_string_has_suffix(const char *s, const char *suffix);

/**
 * Replace all occurrences of a substring.
 *
 * @param s The original string
 * @param old The substring to replace
 * @param new The replacement
 * @return The new string (must be freed with foundation_free_string)
 */
char *foundation_string_replace(const char *s, const char *old, const char *new);

/* ======================== */
/* Byte Count Formatting    */
/* ======================== */

/**
 * Format a byte count as a human-readable string.
 *
 * @param bytes The byte count
 * @return The formatted string (must be freed with foundation_free_string)
 */
char *foundation_format_byte_count(int64_t bytes);

/* ======================== */
/* Memory Management        */
/* ======================== */

/**
 * Free a string previously returned by a foundation_* function.
 *
 * @param ptr The string to free
 */
void foundation_free_string(char *ptr);

/* ======================== */
/* Version                  */
/* ======================== */

/**
 * Get the library version string.
 *
 * @return The version string (do NOT free)
 */
const char *foundation_version(void);

#ifdef __cplusplus
}
#endif

#endif /* TONTOO_FOUNDATION_H */
