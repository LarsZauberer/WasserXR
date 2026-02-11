#include "TheSeed/ecs/Scene.h"
#include <stdio.h>

int ts_select_ts_mesh_renderer(TS_Scene_t *scene, const size_t entity) {
  if (ts_entity_get_component(scene, entity, "TS_Transform") &&
      ts_entity_get_component(scene, entity, "TS_Mesh")) {
    return 1;
  } else {
    return 0;
  }
}

void ts_system_ts_mesh_renderer(TS_Scene_t *scene, size_t *entities, size_t n) {
  printf("Bhhh: %ld\n", n);
  return;
}
