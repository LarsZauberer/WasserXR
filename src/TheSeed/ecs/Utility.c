#include "Scene_internal.h"

int ts_default_selector(TS_Scene *scene, const TS_Entity entity_id) {
  return 0;
}

int ts_compare_systems_priority(gconstpointer left, gconstpointer right) {
  const TS_System_Handler *system_a = *(const TS_System_Handler **)left;
  const TS_System_Handler *system_b = *(const TS_System_Handler **)right;

  return system_a->priority - system_b->priority;
}
