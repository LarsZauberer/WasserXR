#include "TheSeed/ecs/Scene.h"
#include "glib.h"
#include <TheSeed/ecs/logging.h>

typedef struct TestCase TestCase;

struct TestCase {
  TS_Scene *scene;
  int should_succeed;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;
  if (input->scene) {
    ts_destroy_scene(input->scene);
  }
}

static void unittest(const void *ptr) {
  const TestCase *input = ptr;
  
  if (!input->should_succeed) {
    // For NULL scene, we can't really test much as behavior is undefined
    return;
  }
  
  // Add multiple entities and verify they have unique IDs
  TS_Entity entity1 = ts_add_entity(input->scene);
  TS_Entity entity2 = ts_add_entity(input->scene);
  TS_Entity entity3 = ts_add_entity(input->scene);
  
  // Check that entities have different IDs
  ts_assert(entity1 != entity2, "Entity IDs should be unique");
  ts_assert(entity2 != entity3, "Entity IDs should be unique");
  ts_assert(entity1 != entity3, "Entity IDs should be unique");
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  ts_logging_init(TS_LOG_DEBUG);
  ts_add_logger(ts_stdout_logger);

  // Constructing Cases
  TS_Scene *valid_scene = ts_create_scene();

  TestCase cases[] = {
      {NULL, 0},
      {valid_scene, 1},
  };

  // Constructing Tests
  for (size_t i = 0; i < 2; i++) {
    char *path = g_strdup_printf("/theseed/test_ecs_add_entity/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
