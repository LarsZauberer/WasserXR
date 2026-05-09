#include "WasserXR/ecs/logging.h"
#include "WasserXR/ecs/utils.h"
#include <glib.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

#define WXR_FUZZY_LENGTH 1000

static void test_copy_char_on_stack() {
  char *stack = "Hello World!";
  char *copy = wxr_copy_char_ptr(stack);
  g_assert(copy);
  g_assert(strcmp(stack, copy) == 0);
  g_assert(stack != copy);
  free(copy);
}

static void test_copy_char_fuzzy() {
  for (int i = 0; i < WXR_FUZZY_LENGTH; i++) {
    int length = g_test_rand_int_range(1, 1000);
    char *input = (char *)malloc(length + 1);

    // Fill with random characters
    for (int j = 0; j < length; j++) {
      input[j] = (char)g_test_rand_int_range(32, 126); // printable ASCII
    }
    input[length] = '\0';

    char *copy = wxr_copy_char_ptr(input);
    g_assert(copy);
    g_assert(strcmp(input, copy) == 0);
    g_assert(copy[length] == '\0');
    free(copy);
    free(input);
  }
}

static void test_copy_char_null() {
  char *out = wxr_copy_char_ptr(NULL);
  g_assert(!out); // Check that the output is null
}

static void test_len_till_null() {
  char *src = "Hello World!";

  g_assert(wxr_len_till_null(src, sizeof(char)) == 12);
}

static void test_copy_till_null() {
  char *src = "Hello World!";
  char *dest = wxr_memcpy_till_null(src, sizeof(char));
  g_assert(src != dest);
  g_assert(strcmp(src, dest) == 0);
  free(dest);
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  g_test_add_func("/wasserxr/test_copy_char_on_stack", test_copy_char_on_stack);
  g_test_add_func("/wasserxr/test_copy_char_fuzzy", test_copy_char_fuzzy);
  g_test_add_func("/wasserxr/test_copy_char_null", test_copy_char_null);
  g_test_add_func("/wasserxr/test_len_till_null", test_len_till_null);
  g_test_add_func("/wasserxr/test_copy_till_null", test_copy_till_null);

  return g_test_run();
}
