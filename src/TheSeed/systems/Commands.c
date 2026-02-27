// This is not a System
#include "TheSeed/systems/Commands.h"
#include "TheSeed/core/logging.h"
#include "TheSeed/ecs/Scene.h"
#include <stdlib.h>
#include <string.h>

TS_Command *ts_create_command_list(size_t *size) {
  *size = 3;
  TS_Command *command_list = (TS_Command *)malloc(sizeof(TS_Command) * *size);

  command_list[0] = (TS_Command){"reload", ts_command_reload};
  command_list[1] = (TS_Command){"exit", ts_command_exit};
  command_list[2] = (TS_Command){"removeEntity", ts_command_removeEntity};

  return command_list;
}

void ts_destroy_command_list(TS_Command *ptr) { free(ptr); }

void ts_command_reload(char **args, TS_Scene *scene) {
  ts_set_scene_reload(scene);
}

static int ts_get_window(TS_Scene *scene, TS_Entity entity) {
  if (ts_entity_get_component(scene, entity, "TS_Window")) {
    return 1;
  }
  return 0;
}

void ts_command_exit(char **args, TS_Scene *scene) {
  size_t num_entities = 0;
  TS_Entity *entities = ts_find_entities_with_selector_and_groups(
      &num_entities, scene, ts_get_window, 1);
  for (size_t i = 0; i < num_entities; i++) {
    ts_remove_entity(scene, entities[i]);
  }
  free(entities);
}

void ts_command_removeEntity(char **args, TS_Scene *scene) {
  if (!*args) {
    ts_warn("Remove Entity requires the entity id to remove");
    return;
  }
  size_t entity_id = strtol(args[0], NULL, 10);
  ts_remove_entity(scene, entity_id);
}
