#include "TheSeed/core/utils.h"
#include <stddef.h>
#include <stdlib.h>
#include <string.h>

char *ts_copy_char_ptr(const char *src) {
  size_t path_length = strlen(src);
  char *out = (char *)malloc(sizeof(char) * (path_length + 1));
  strcpy(out, src);
  return out;
}
