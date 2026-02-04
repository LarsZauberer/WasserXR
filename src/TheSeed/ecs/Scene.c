#include "TheSeed/ecs/Scene.h"
#include "TheSeed/core/utils.h"
#include <dlfcn.h>
#include <glib.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
  char *path;
  void *fd;
} TS_Loaded_Plugin;

typedef struct {
  char *id;
  size_t entity;
  TS_Loaded_Plugin *plugin;
  void *component;
} TS_Component_Handler;

struct TS_Scene_t {
  GArray *plugins;
  size_t entity_counter;
  GArray *entities;
  GArray *components;
};

TS_Scene_t *ts_create_scene() {
  TS_Scene_t *p = (TS_Scene_t *)malloc(sizeof(TS_Scene_t));
  p->plugins = g_array_new(FALSE, FALSE, sizeof(TS_Loaded_Plugin *));
  p->entities = g_array_new(FALSE, FALSE, sizeof(size_t));
  p->entity_counter = 0;
  p->components = g_array_new(FALSE, FALSE, sizeof(TS_Component_Handler *));
  return p;
}

void ts_destroy_scene(TS_Scene_t *scene) {
  size_t plugins_len = scene->plugins->len;
  for (size_t i = 0; i < plugins_len; i++) {
    const TS_Loaded_Plugin *plugin =
        g_array_index(scene->plugins, TS_Loaded_Plugin *, 0);
    ts_unload_plugin(scene, plugin->path);
  }
  size_t entities_len = scene->entities->len;
  for (size_t i = 0; i < entities_len; i++) {
    const size_t entity = g_array_index(scene->entities, size_t, 0);
    // This will also destroy all the components associated with the entity
    ts_remove_entity(scene, entity);
  }
  g_array_free(scene->plugins, TRUE);
  g_array_free(scene->entities, TRUE);
  g_array_free(scene->components, TRUE);

  free(scene);
  return;
}

size_t ts_add_entity(TS_Scene_t *scene) {
  size_t entity = scene->entity_counter;
  scene->entity_counter += 1;
  g_array_append_val(scene->entities, entity);
  return entity;
}

static long ts_get_entity_index(const TS_Scene_t *scene, const size_t entity) {
  for (size_t i = 0; i < scene->entities->len; i++) {
    const size_t e = g_array_index(scene->entities, size_t, i);
    if (e == entity) {
      return i;
    }
  }
  return -1;
}

int ts_remove_entity(TS_Scene_t *scene, const size_t entity) {
  long index = ts_get_entity_index(scene, entity);
  if (index == -1L) {
    return 1;
  }
  g_array_remove_index(scene->entities, index);

  // Cleanup the components
  for (size_t i = 0; i < scene->components->len; i++) {
    TS_Component_Handler *c =
        g_array_index(scene->components, TS_Component_Handler *, i);
    if (c->entity == entity) {
      ts_remove_component(scene, entity, c->id);
      i--;
    }
  }

  return 1;
}

int ts_load_plugin(TS_Scene_t *scene, const char *path) {
  // TODO: Add check if the plugin is already loaded
  TS_Loaded_Plugin *plugin =
      (TS_Loaded_Plugin *)malloc(sizeof(TS_Loaded_Plugin));

  plugin->path = ts_copy_char_ptr(path);

  plugin->fd = dlopen(path, RTLD_NOW);
  if (!plugin->fd) {
    free(plugin->path);
    free(plugin);
  }
  g_array_append_val(scene->plugins, plugin);
  return 0;
}

static long ts_get_plugin_index(const TS_Scene_t *scene, const char *path) {
  for (size_t i = 0; i < scene->plugins->len; i++) {
    const TS_Loaded_Plugin *plugin =
        g_array_index(scene->plugins, TS_Loaded_Plugin *, i);
    if (strcmp(plugin->path, path) == 0) {
      return i;
    }
  }
  return -1;
}

int ts_unload_plugin(TS_Scene_t *scene, const char *path) {
  long index = ts_get_plugin_index(scene, path);
  if (index == -1L) {
    return 1;
  }

  TS_Loaded_Plugin *plugin =
      g_array_index(scene->plugins, TS_Loaded_Plugin *, index);
  dlclose(plugin->fd);
  free(plugin->path);
  free(plugin);
  g_array_remove_index(scene->plugins, index);
  return 0;
}

int ts_add_component(TS_Scene_t *scene, const size_t entity, const char *id) {
  // Check if the entity exists
  long does_exist = ts_get_entity_index(scene, entity);
  if (does_exist == -1) {
    return 0;
  }

  GString *gstring_id = g_string_new(id);

  TS_Loaded_Plugin *plugin;
  void *(*create_func)(void);

  for (size_t i = 0; i < scene->plugins->len; i++) {
    // Try to find a function that has the suitable naming
    plugin = g_array_index(scene->plugins, TS_Loaded_Plugin *, i);
    GString *gstring_id_cpy = g_string_copy(gstring_id);
    g_string_prepend(gstring_id_cpy, "ts_create_");
    create_func = dlsym(plugin->fd, gstring_id_cpy->str);
    g_string_free(gstring_id_cpy, TRUE);

    if (create_func) {
      break;
    }
  }

  g_string_free(gstring_id, TRUE);

  if (!create_func) {
    // Not found the constructor for the component
    return 1;
  }

  // Create the component handler object
  TS_Component_Handler *component_handler =
      (TS_Component_Handler *)malloc(sizeof(TS_Component_Handler));

  component_handler->id = ts_copy_char_ptr(id);
  component_handler->entity = entity;
  component_handler->plugin = plugin;

  void *component = create_func();

  component_handler->component = component;

  // Add the component
  g_array_append_val(scene->components, component_handler);

  return 0;
}

static long ts_get_component_index_from_entity_and_id(TS_Scene_t *scene,
                                                      const size_t entity,
                                                      const char *id) {
  // Entity and id can uniquely identify a component
  for (size_t i = 0; i < scene->components->len; i++) {
    const TS_Component_Handler *component =
        g_array_index(scene->components, TS_Component_Handler *, i);
    if (strcmp(component->id, id) == 0 && component->entity == entity) {
      return i;
    }
  }
  return -1L;
}

int ts_remove_component(TS_Scene_t *scene, const size_t entity,
                        const char *id) {
  // There can only be one association between the entity and the component
  long index = ts_get_component_index_from_entity_and_id(scene, entity, id);
  if (index == -1L) {
    return 1;
  }

  TS_Component_Handler *component =
      g_array_index(scene->components, TS_Component_Handler *, index);
  // This is the one to remove
  g_array_remove_index(scene->components, index);
  free(component->id);
  free(component->component);
  free(component);
  return 0;
}

// Debug Functions

void ts_print_entities(TS_Scene_t *scene) {
  printf("Active Entities %d:\n", scene->entities->len);
  for (size_t i = 0; i < scene->entities->len; i++) {
    const size_t entity = g_array_index(scene->entities, size_t, i);
    printf("- Entity %ld\n", entity);
  }
}

void ts_print_plugins(TS_Scene_t *scene) {
  printf("Loaded Plugins %d:\n", scene->plugins->len);
  for (size_t i = 0; i < scene->plugins->len; i++) {
    const TS_Loaded_Plugin *plugin =
        g_array_index(scene->plugins, TS_Loaded_Plugin *, i);
    printf("- %s\n", plugin->path);
  }
}

void ts_print_components(TS_Scene_t *scene) {
  printf("Active Components %d:\n", scene->components->len);
  for (size_t i = 0; i < scene->entities->len; i++) {
    const size_t entity = g_array_index(scene->entities, size_t, i);
    printf("- Entity %ld:\n", entity);
    for (size_t j = 0; j < scene->components->len; j++) {
      const TS_Component_Handler *component =
          g_array_index(scene->components, TS_Component_Handler *, j);
      if (component->entity == entity) {
        printf("  - %s (%s)\n", component->id, component->plugin->path);
      }
    }
  }
}
