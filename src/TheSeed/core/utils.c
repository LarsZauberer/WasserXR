#include "TheSeed/core/utils.h"
#include <glib.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

char *ts_copy_char_ptr(const char *src) {
  size_t path_length = strlen(src);
  char *out = (char *)malloc(sizeof(char) * (path_length + 1));
  strcpy(out, src);
  return out;
}

char *ts_read_file(const char *filepath) {
  FILE *file = fopen(filepath, "r");
  if (!file) {
    fprintf(stderr, "Error: Failed to open file '%s'\n", filepath);
    return NULL;
  }

  GString *gstring = g_string_new(NULL);

  char buffer[1024];
  while (fgets(buffer, sizeof(buffer), file)) {
    g_string_append(gstring, buffer);
  }

  fclose(file);

  // Extract the C string and free only the GString wrapper
  return g_string_free(gstring, FALSE);
}
