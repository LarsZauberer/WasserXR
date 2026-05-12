/*
 * Copyright (c) 2026 Ian Wasser
 * Licensed under the WasserXR License.
 * You may not use this file except in compliance with the License.
 * See LICENSE.md for details.
 */

#include <stddef.h>

char *wxr_copy_char_ptr(const char *src);

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
char *wxr_read_file(const char *filepath);

size_t wxr_len_till_null(const void *data, size_t element_size);

void *wxr_memcpy_till_null(const void *data, size_t element_size);
