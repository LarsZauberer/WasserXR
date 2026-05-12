/*
 * Copyright (c) 2026 Ian Wasser
 * Licensed under the WasserXR License.
 * You may not use this file except in compliance with the License.
 * See LICENSE.md for details.
 */

#include "Scene_internal.h"
#include "WasserXR/ecs/Scene.h"
#include <WasserXR/ecs/logging.h>
#include <string.h>

long wxr_get_entity_index(const WXR_Scene *scene, const WXR_Entity entity) {
  for (long i = 0; i < scene->entities->len; i++) {
    const WXR_Entity eentity = g_array_index(scene->entities, WXR_Entity, i);
    if (eentity == entity) {
      wxr_assert(scene, "Scene is NULL during wxr_get_entity_index");
      return i;
    }
  }
  return -1;
}

long wxr_get_plugin_index(const WXR_Scene *scene, const char *path) {
  wxr_assert(scene, "Scene is NULL during wxr_get_plugin_index");
  for (long i = 0; i < scene->plugins->len; i++) {
    const WXR_Plugin_Handler *plugin =
        g_array_index(scene->plugins, WXR_Plugin_Handler *, i);
    if (strcmp(plugin->path, path) == 0) {
      return i;
    }
  }
  return -1;
}

long wxr_get_system_index(const WXR_Scene *scene, const char *system_id) {
  wxr_assert(scene, "Scene is NULL during ts_get_system_index_from_id");
  wxr_assert(system_id, "Id is NULL during ts_get_system_index_from_id");
  for (long i = 0; i < scene->systems->len; i++) {
    const WXR_System_Handler *system =
        g_array_index(scene->systems, WXR_System_Handler *, i);
    if (strcmp(system->id, system_id) == 0) {
      return i;
    }
  }
  return -1L;
}

long wxr_get_component_index(const WXR_Scene *scene, const WXR_Entity entity,
                             const char *component_id) {
  wxr_assert_abort_value(scene, -1,
                         "Scene is NULL during wxr_get_component_index");
  wxr_assert_abort_value(component_id, -1,
                         "Component ID is NULL during wxr_get_component_index");
  wxr_assert_abort_value(entity < scene->entity_counter, -1,
                         "Entity is invalid during wxr_get_component_index");
  for (long i = 0; i < scene->components->len; i++) {
    const WXR_Component_Handler *handler =
        g_array_index(scene->components, WXR_Component_Handler *, i);
    if (strcmp(component_id, handler->id) == 0 && handler->entity == entity) {
      return i;
    }
  }
  return -1;
}

WXR_Component_Handler *wxr_find_handler_for_component(const WXR_Scene *scene,
                                                      const void *component) {
  for (size_t i = 0; i < scene->components->len; i++) {
    WXR_Component_Handler *handler =
        g_array_index(scene->components, WXR_Component_Handler *, i);
    if (handler->component == component) {
      return handler;
    }
  }

  return NULL;
}
