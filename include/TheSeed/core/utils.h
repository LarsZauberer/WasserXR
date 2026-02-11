#include <glib.h>

char *ts_copy_char_ptr(const char *);

int ts_read_file_to_gstring(const char *filepath, GString **out_string);
