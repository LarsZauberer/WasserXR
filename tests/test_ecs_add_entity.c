#include "WasserXR/ecs/Scene.h"
#include "glib.h"
#include <WasserXR/ecs/logging.h>

typedef struct TestCase TestCase;

struct TestCase {
  WXR_Scene *scene;
  int should_succeed;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;
  if (input->scene) {
    wxr_destroy_scene(input->scene);
  }
}

static void unittest(const void *ptr) {
  const TestCase *input = ptr;
  
  if (!input->should_succeed) {
    // For NULL scene, we can't really test much as behavior is undefined
    return;
  }
  
  // Add multiple entities and verify they have unique IDs
  WXR_Entity entity1 = wxr_add_entity(input->scene);
  WXR_Entity entity2 = wxr_add_entity(input->scene);
  WXR_Entity entity3 = wxr_add_entity(input->scene);
  
  // Check that entities have different IDs
  wxr_assert(entity1 != entity2, "Entity IDs should be unique");
  wxr_assert(entity2 != entity3, "Entity IDs should be unique");
  wxr_assert(entity1 != entity3, "Entity IDs should be unique");
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  wxr_logging_init(WXR_LOG_DEBUG);
  wxr_add_logger(wxr_stdout_logger);

  // Constructing Cases
  WXR_Scene *valid_scene = wxr_create_scene();

  TestCase cases[] = {
      {NULL, 0},
      {valid_scene, 1},
  };

  // Constructing Tests
  for (size_t i = 0; i < 2; i++) {
    char *path = g_strdup_printf("/wasserxr/test_ecs_add_entity/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
