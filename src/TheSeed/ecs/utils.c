#include "TheSeed/ecs/utils.h"
#include "TheSeed/ecs/logging.h"
#include <glib.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

char *ts_copy_char_ptr(const char *src) {
  if (!src) {
    return NULL;
  }
  size_t path_length = strlen(src);
  char *out = (char *)malloc(sizeof(char) * (path_length + 1));
  g_strlcpy(out, src, strlen(src) + 1);
  return out;
}

char *ts_read_file(const char *filepath) {
  FILE *file = fopen(filepath, "r");
  if (!file) {
    ts_error("Error: Failed to open file '%s'", filepath);
    return NULL;
  }

  GString *gstring = g_string_new(NULL);

  char buffer[1024];
  while (fgets(buffer, sizeof(buffer), file)) {
    g_string_append(gstring, buffer);
  }

  int close_status = fclose(file);
  if (!close_status) {
    ts_assert_recoverable(!close_status, "Failed to close the file `%s`",
                          filepath);
  }

  // Extract the C string and free only the GString wrapper
  return g_string_free(gstring, FALSE);
}

size_t ts_len_till_null(const void *data, size_t element_size) {
  if (!data) {
    return 0;
  }
  size_t counter = 0;
  const unsigned char *ptr = (const unsigned char *)data;
  while (1) {
    // Check if all bytes in the current element are zero
    int is_null = 1;
    for (size_t i = 0; i < element_size; i++) {
      if (ptr[i] != 0) {
        is_null = 0;
        break;
      }
    }

    if (!is_null) {
      counter++;
      ptr += element_size;
    } else {
      break;
    }
  }
  return counter;
}

void *ts_memcpy_till_null(const void *data, size_t element_size) {
  if (!data) {
    return NULL;
  }

  // Get the number of elements (excluding null terminator)
  size_t count = ts_len_till_null(data, element_size);

  // Allocate memory for count elements + 1 null terminator
  size_t total_size = (count + 1) * element_size;
  void *copy = malloc(total_size);

  if (!copy) {
    ts_error("Error: Failed to allocate memory in ts_memcpy_till_null");
    return NULL;
  }

  // Copy all elements including the null terminator
  memcpy(copy, data, total_size);

  return copy;
}
