char *ts_copy_char_ptr(const char *src);

/**
 * Reads the contents of a file into a dynamically allocated string.
 *
 * @param filepath Path to the file to read
 * @return Pointer to a null-terminated string containing the file contents,
 *         or NULL if the file could not be opened or read.
 *         The caller is responsible for freeing the returned string using
 * free().
 *
 * Note: On failure, an error message is printed to stdout.
 */
char *ts_read_file(const char *filepath);
