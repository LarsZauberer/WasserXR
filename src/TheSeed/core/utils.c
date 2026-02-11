#include "TheSeed/core/utils.h"
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

int ts_read_file_to_gstring(const char *filepath, GString **out_string) {
  FILE *file = fopen(filepath, "r");
  if (!file) {
    fprintf(stderr, "Error: Failed to open file '%s'\n", filepath);
    return 1;
  }

  *out_string = g_string_new(NULL);

  char buffer[1024];
  while (fgets(buffer, sizeof(buffer), file)) {
    g_string_append(*out_string, buffer);
  }

  fclose(file);
  return 0;
}
