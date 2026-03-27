#include "TheSeed/ecs/Scene.h"
#include "glib.h"
#include <TheSeed/core/logging.h>
#include <stdlib.h>

typedef struct TestCase TestCase;

struct TestCase {
  TS_Scene *scene;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;
  if (input->scene) {
    ts_destroy_scene(input->scene);
  }
}

static void unittest(const void *ptr) {
  const TestCase *input = ptr;
  
  // Add entities before reset
  ts_add_entity(input->scene);
  ts_add_entity(input->scene);
  
  // Reset the scene
  ts_reset_scene(input->scene);
  
  // Check that entities are cleared
  size_t count = 0;
  TS_Entity *entities = ts_get_entities(&count, input->scene);
  ts_assert(count == 0, "Scene should have no entities after reset");
  
  if (entities) {
    free(entities);
  }
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  ts_logging_init(TS_LOG_DEBUG);
  ts_add_logger(ts_stdout_logger);

  // Constructing Cases
  TS_Scene *test_scene = ts_create_scene();

  TestCase cases[] = {
      {test_scene},
  };

  // Constructing Tests
  for (size_t i = 0; i < 1; i++) {
    char *path = g_strdup_printf("/theseed/test_ecs_reset_scene/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
