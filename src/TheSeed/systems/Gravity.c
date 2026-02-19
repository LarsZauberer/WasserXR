#include "TheSeed/components/Transform.h"
#include "TheSeed/ecs/Scene.h"
#include <stdio.h>

int ts_select_TS_Gravity(TS_Scene *scene, const TS_Entity entity) {
  if (ts_entity_get_component(scene, entity, "TS_Transform")) {
    return 1;
  }
  return 0;
}

void ts_system_TS_Gravity(TS_Scene *scene, const TS_Entity *entities,
                          size_t n) {
  for (size_t i = 0; i < n; i++) {
    TS_Transform_t *t = (TS_Transform_t *)ts_entity_get_component(
        scene, entities[i], "TS_Transform");
    t->position[2] += -1;
  }
}

int ts_select_TS_Print_Transform(TS_Scene *scene, const TS_Entity entity) {
  if (ts_entity_get_component(scene, entity, "TS_Transform")) {
    return 1;
  }
  return 0;
}

void ts_system_TS_Print_Transform(TS_Scene *scene, const TS_Entity *entities,
                                  size_t n) {
  for (size_t i = 0; i < n; i++) {
    TS_Transform_t *t = (TS_Transform_t *)ts_entity_get_component(
        scene, entities[i], "TS_Transform");
    printf("Entity %ld: x: %f y: %f z: %f\n", entities[i], t->position[0],
           t->position[1], t->position[2]);
  }
}
