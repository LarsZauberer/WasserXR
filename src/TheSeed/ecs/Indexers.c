#include "Scene_internal.h"
#include <TheSeed/core/logging.h>

long ts_get_entity_index(const TS_Scene *scene, const TS_Entity entity) {
  for (long i = 0; i < scene->entities->len; i++) {
    const TS_Entity eentity = g_array_index(scene->entities, TS_Entity, i);
    if (eentity == entity) {
      ts_assert(scene, "Scene is NULL during ts_get_entity_index");
      return i;
    }
  }
  return -1;
}

long ts_get_plugin_index(const TS_Scene *scene, const char *path) {
  ts_assert(scene, "Scene is NULL during ts_get_plugin_index");
  for (long i = 0; i < scene->plugins->len; i++) {
    const TS_Plugin_Handler *plugin =
        g_array_index(scene->plugins, TS_Plugin_Handler *, i);
    if (strcmp(plugin->path, path) == 0) {
      return i;
    }
  }
  return -1;
}

long ts_get_system_index_from_id(TS_Scene *scene, const char *system_id) {
  ts_assert(scene, "Scene is NULL during ts_get_system_index_from_id");
  ts_assert(system_id, "Id is NULL during ts_get_system_index_from_id");
  for (long i = 0; i < scene->systems->len; i++) {
    const TS_System_Handler *system =
        g_array_index(scene->systems, TS_System_Handler *, i);
    if (strcmp(system->id, system_id) == 0) {
      return i;
    }
  }
  return -1L;
}

TS_Component_Handler *ts_find_handler_for_component(TS_Scene *scene,
                                                    void *component) {
  for (size_t i = 0; i < scene->components->len; i++) {
    TS_Component_Handler *handler =
        g_array_index(scene->components, TS_Component_Handler *, i);
    if (handler->component == component) {
      return handler;
    }
  }

  return NULL;
}
