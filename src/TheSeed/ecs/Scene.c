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
  TS_Entity entity;
  TS_Loaded_Plugin *plugin;
  TS_Component_Destroyer destroyer;
  void *component;
} TS_Component_Handler;

typedef struct {
  char *id;
  int priority;
  int active;
  TS_Loaded_Plugin *plugin;
  TS_System_Selector selector;
  TS_System_Function system;
} TS_System_Handler;

struct TS_Scene_t {
  GArray *plugins;
  TS_Entity entity_counter;
  GArray *entities;
  GArray *components;
  GArray *systems;
  int should_reload;
};

TS_Scene_t *ts_create_scene() {
  TS_Scene_t *p = (TS_Scene_t *)malloc(sizeof(TS_Scene_t));
  p->plugins = g_array_new(FALSE, FALSE, sizeof(TS_Loaded_Plugin *));
  p->entities = g_array_new(FALSE, FALSE, sizeof(TS_Entity));
  p->entity_counter = 0;
  p->components = g_array_new(FALSE, FALSE, sizeof(TS_Component_Handler *));
  p->systems = g_array_new(FALSE, FALSE, sizeof(TS_System_Handler *));
  p->should_reload = 0;
  return p;
}

void ts_destroy_scene(TS_Scene_t *scene) {
  // Unloading all plugins
  // This will result in destroying all the components and systems with it.
  size_t plugins_len = scene->plugins->len;
  for (size_t i = 0; i < plugins_len; i++) {
    const TS_Loaded_Plugin *plugin =
        g_array_index(scene->plugins, TS_Loaded_Plugin *, 0);
    ts_unload_plugin(scene, plugin->path);
  }
  // Clean up all the rest of the entities
  size_t entities_len = scene->entities->len;
  for (size_t i = 0; i < entities_len; i++) {
    const TS_Entity entity = g_array_index(scene->entities, TS_Entity, 0);
    // This will also destroy all the components associated with the entity
    ts_remove_entity(scene, entity);
  }
  g_array_free(scene->plugins, TRUE);
  g_array_free(scene->entities, TRUE);
  g_array_free(scene->components, TRUE);
  g_array_free(scene->systems, TRUE);

  free(scene);
  return;
}

TS_Entity ts_add_entity(TS_Scene_t *scene) {
  TS_Entity entity = scene->entity_counter;
  scene->entity_counter += 1;
  g_array_append_val(scene->entities, entity);
  return entity;
}

static long ts_get_entity_index(const TS_Scene_t *scene,
                                const TS_Entity entity) {
  for (size_t i = 0; i < scene->entities->len; i++) {
    const TS_Entity e = g_array_index(scene->entities, TS_Entity, i);
    if (e == entity) {
      return i;
    }
  }
  return -1;
}

int ts_remove_entity(TS_Scene_t *scene, const TS_Entity entity) {
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

int ts_load_plugin(TS_Scene_t *scene, const char *path) {
  long does_exist = ts_get_plugin_index(scene, path);
  if (does_exist != -1L) {
    return 1;
  }
  TS_Loaded_Plugin *plugin =
      (TS_Loaded_Plugin *)malloc(sizeof(TS_Loaded_Plugin));

  plugin->path = ts_copy_char_ptr(path);

  plugin->fd = dlopen(path, RTLD_NOW);
  if (!plugin->fd) {
    printf("%s\n", dlerror());
    free(plugin->path);
    free(plugin);
    return 1;
  }
  g_array_append_val(scene->plugins, plugin);
  return 0;
}

int ts_unload_plugin(TS_Scene_t *scene, const char *path) {
  long index = ts_get_plugin_index(scene, path);
  if (index == -1L) {
    return 1;
  }

  // Destroy all systems associated to the plugin
  for (size_t i = 0; i < scene->systems->len; i++) {
    TS_System_Handler *system =
        g_array_index(scene->systems, TS_System_Handler *, i);
    if (strcmp(system->plugin->path, path) == 0) {
      ts_remove_system(scene, system->id);
      i--;
    }
  }

  // Destroy all the components associated to the plugin
  for (size_t i = 0; i < scene->components->len; i++) {
    TS_Component_Handler *component =
        g_array_index(scene->components, TS_Component_Handler *, i);
    if (strcmp(component->plugin->path, path) == 0) {
      ts_remove_component(scene, component->entity, component->id);
      i--;
    }
  }

  // Destroy the plugins
  TS_Loaded_Plugin *plugin =
      g_array_index(scene->plugins, TS_Loaded_Plugin *, index);
  dlclose(plugin->fd);
  free(plugin->path);
  free(plugin);
  g_array_remove_index(scene->plugins, index);

  return 0;
}

int ts_add_component(TS_Scene_t *scene, const TS_Entity entity,
                     const char *id) {
  // Check if the entity exists
  long does_exist = ts_get_entity_index(scene, entity);
  if (does_exist == -1) {
    return 1;
  }

  GString *symbol_id = g_string_new(id);

  TS_Loaded_Plugin *plugin;
  TS_Component_Creator creator;
  TS_Component_Destroyer destroyer;

  for (size_t i = 0; i < scene->plugins->len; i++) {
    plugin = g_array_index(scene->plugins, TS_Loaded_Plugin *, i);

    // Try to find a function that has the suitable naming
    GString *creator_symbol_id = g_string_copy(symbol_id);
    GString *destroyer_symbol_id = g_string_copy(symbol_id);
    g_string_prepend(creator_symbol_id, CREATOR_FUNCION_PREFIX);
    g_string_prepend(destroyer_symbol_id, DESTROYER_FUNCION_PREFIX);

    creator = dlsym(plugin->fd, creator_symbol_id->str);
    g_string_free(creator_symbol_id, TRUE);

    destroyer = dlsym(plugin->fd, destroyer_symbol_id->str);
    g_string_free(destroyer_symbol_id, TRUE);

    if (creator && destroyer) {
      break;
    }
  }

  g_string_free(symbol_id, TRUE);

  if (!creator || !destroyer) {
    // Not found the constructor or destroyer for the component
    return 1;
  }

  // Create the component handler object
  TS_Component_Handler *component_handler =
      (TS_Component_Handler *)malloc(sizeof(TS_Component_Handler));

  component_handler->id = ts_copy_char_ptr(id);
  component_handler->entity = entity;
  component_handler->plugin = plugin;
  component_handler->destroyer = destroyer;

  void *component = creator();

  component_handler->component = component;

  // Add the component
  g_array_append_val(scene->components, component_handler);

  return 0;
}

static long ts_get_component_index_from_entity_and_id(TS_Scene_t *scene,
                                                      const TS_Entity entity,
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

void *ts_entity_get_component(TS_Scene_t *scene, const TS_Entity entity,
                              const char *id) {
  long index = ts_get_component_index_from_entity_and_id(scene, entity, id);
  if (index == -1L) {
    return NULL;
  }

  TS_Component_Handler *component =
      g_array_index(scene->components, TS_Component_Handler *, index);
  return component->component;
}

int ts_remove_component(TS_Scene_t *scene, const TS_Entity entity,
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
  component->destroyer(
      component->component); // Call the destroyer for the component
  // The component pointer itself should be destroyed by the destroyer function
  free(component);
  return 0;
}

static long ts_get_system_index_from_id(TS_Scene_t *scene, const char *id) {
  for (size_t i = 0; i < scene->systems->len; i++) {
    const TS_System_Handler *system =
        g_array_index(scene->systems, TS_System_Handler *, i);
    if (strcmp(system->id, id) == 0) {
      return i;
    }
  }
  return -1L;
}

int ts_add_system(TS_Scene_t *scene, const char *id, int priority) {
  long index = ts_get_system_index_from_id(scene, id);
  if (index != -1L) {
    // Already exists, won't add
    return 1;
  }

  // Find the selector and system function

  // Working string
  GString *symbol = g_string_new(id);
  TS_Loaded_Plugin *plugin;
  TS_System_Selector selector;
  TS_System_Function system;

  for (size_t i = 0; i < scene->plugins->len; i++) {
    // Try to find a function that has the suitable naming
    plugin = g_array_index(scene->plugins, TS_Loaded_Plugin *, i);
    GString *selector_symbol_cpy = g_string_copy(symbol);
    g_string_prepend(selector_symbol_cpy, SYSTEM_SELECTOR_PREFIX);
    selector = dlsym(plugin->fd, selector_symbol_cpy->str);
    GString *system_symbol_cpy = g_string_copy(symbol);
    g_string_prepend(system_symbol_cpy, SYSTEM_FUNCTION_PREFIX);
    system = dlsym(plugin->fd, system_symbol_cpy->str);
    g_string_free(selector_symbol_cpy, TRUE);
    g_string_free(system_symbol_cpy, TRUE);

    if (selector && system) {
      break;
    }
  }

  if (!selector || !system) {
    // Didn't find the selector
    g_string_free(symbol, TRUE);
    return 1;
  }

  // Found everything -> We can build the system handler
  TS_System_Handler *system_handler =
      (TS_System_Handler *)malloc(sizeof(TS_System_Handler));
  system_handler->id = g_string_free(symbol, FALSE);
  system_handler->active = 1;
  system_handler->priority = priority;
  system_handler->system = system;
  system_handler->selector = selector;
  system_handler->plugin = plugin;
  g_array_append_val(scene->systems, system_handler);

  // Sort for priority
  ts_sort_systems(scene);

  return 0;
}

static GArray *ts_find_entities_with_selector(TS_Scene_t *scene,
                                              TS_System_Selector selector) {
  GArray *res = g_array_new(FALSE, FALSE, sizeof(TS_Entity));
  for (size_t i = 0; i < scene->entities->len; i++) {
    TS_Entity entity = g_array_index(scene->entities, TS_Entity, i);
    if (selector(scene, entity)) {
      g_array_append_val(res, entity);
    }
  }
  return res;
}

void ts_tick_scene(TS_Scene_t *scene) {
  for (size_t i = 0; i < scene->systems->len; i++) {
    TS_System_Handler *system =
        g_array_index(scene->systems, TS_System_Handler *, i);
    if (!system->active) {
      continue;
    }
    GArray *entities_array =
        ts_find_entities_with_selector(scene, system->selector);
    size_t n = entities_array->len;
    TS_Entity *entities = (TS_Entity *)g_array_free(entities_array, FALSE);
    system->system(scene, entities, n);
    free(entities);
  }

  // Check if the scene should reload
  if (scene->should_reload) {
    scene->should_reload = 0; // Reset
    ts_reload_all_plugins(scene);
  }
}

static int ts_compare_systems_priority(const gconstpointer a,
                                       const gconstpointer b) {
  const TS_System_Handler *system_a = *(const TS_System_Handler **)a;
  const TS_System_Handler *system_b = *(const TS_System_Handler **)b;

  return system_a->priority - system_b->priority;
}

void ts_sort_systems(TS_Scene_t *scene) {
  g_array_sort(scene->systems, ts_compare_systems_priority);
  return;
}

int ts_remove_system(TS_Scene_t *scene, const char *id) {
  long index = ts_get_system_index_from_id(scene, id);

  if (index == -1L) {
    return 1;
  }

  TS_System_Handler *system =
      g_array_index(scene->systems, TS_System_Handler *, index);
  free(system->id);
  free(system);
  g_array_remove_index(scene->systems, index);
  return 0;
}

int ts_reload_plugin(TS_Scene_t *scene, const char *path,
                     const char *new_path) {
  long index = ts_get_plugin_index(scene, path);
  if (index == -1L) {
    return 1;
  }

  TS_Loaded_Plugin *plugin =
      g_array_index(scene->plugins, TS_Loaded_Plugin *, index);

  // Pre Unload operations

  GArray *component_ids_to_reconstruct =
      g_array_new(FALSE, FALSE, sizeof(char *));
  GArray *component_entity_to_reconstruct =
      g_array_new(FALSE, FALSE, sizeof(TS_Entity));

  for (size_t i = 0; i < scene->components->len; i++) {
    TS_Component_Handler *component =
        g_array_index(scene->components, TS_Component_Handler *, i);

    if (component->plugin == plugin) {
      // This component is part of the plugin
      // Unload the component
      // Save everything that is needed for reconstruction

      char *id = ts_copy_char_ptr(component->id);
      g_array_append_val(component_ids_to_reconstruct, id);
      g_array_append_val(component_entity_to_reconstruct, component->entity);

      // Delete the component
      ts_remove_component(scene, component->entity, component->id);
      i--;
    }
  }

  // Loading the new plugin

  // Copy the path and the new_path over since it might be in a systems memory
  // location
  char *p = ts_copy_char_ptr(new_path);
  free(plugin->path);
  plugin->path = p;

  // Close and Check if you can open the new one
  dlclose(plugin->fd);
  void *new_fd = dlopen(plugin->path, RTLD_NOW);
  if (!new_fd) {
    // Failed to open the new plugin -> Abort
    printf("%s\n", dlerror());
    exit(1);
    return 1;
  }
  plugin->fd = new_fd;

  // Post Unload operations -> Recreation of stuff
  int status = 0;

  // Replace all the bindings of the systems
  for (size_t i = 0; i < scene->systems->len; i++) {
    TS_System_Handler *system =
        g_array_index(scene->systems, TS_System_Handler *, i);

    // Check if the same plugin handler is assigned
    if (system->plugin == plugin) {
      GString *id = g_string_new(system->id);
      GString *id_selector = g_string_copy(id);
      GString *id_function = g_string_copy(id);

      g_string_prepend(id_selector, SYSTEM_SELECTOR_PREFIX);
      g_string_prepend(id_function, SYSTEM_FUNCTION_PREFIX);

      TS_System_Selector selector = dlsym(plugin->fd, id_selector->str);
      TS_System_Function function = dlsym(plugin->fd, id_function->str);

      g_string_free(id_selector, TRUE);
      g_string_free(id_function, TRUE);
      g_string_free(id, TRUE);

      if (!selector || !function) {
        // The systems are not defined anymore and hence need to be deleted
        status = 2;
        ts_remove_system(scene, system->id);
        i--;
        continue;
      }

      system->selector = selector;
      system->system = function;
    }
  }

  // Replace all the components
  for (size_t i = 0; i < component_ids_to_reconstruct->len; i++) {
    char *id_to_construct =
        g_array_index(component_ids_to_reconstruct, char *, i);
    TS_Entity entity_to_construct =
        g_array_index(component_entity_to_reconstruct, TS_Entity, i);

    if (ts_add_component(scene, entity_to_construct, id_to_construct)) {
      status = 2;
    }
  }

  // Destroy all the helper arrays
  for (size_t i = 0; i < component_ids_to_reconstruct->len; i++) {
    char *id_to_free = g_array_index(component_ids_to_reconstruct, char *, i);
    free(id_to_free);
  }
  g_array_free(component_ids_to_reconstruct, TRUE);
  g_array_free(component_entity_to_reconstruct, TRUE);

  return status;
}

int ts_reload_all_plugins(TS_Scene_t *scene) {
  GArray *plugins = g_array_copy(scene->plugins);

  for (size_t i = 0; i < plugins->len; i++) {
    TS_Loaded_Plugin *p = g_array_index(plugins, TS_Loaded_Plugin *, i);
    ts_reload_plugin(scene, p->path, p->path);
  }

  g_array_free(plugins, TRUE);
  return 0;
}

void ts_set_scene_reload(TS_Scene_t *scene) {
  scene->should_reload = 1;
  return;
}

// Debug Functions

void ts_print_entities(TS_Scene_t *scene) {
  printf("Active Entities %d:\n", scene->entities->len);
  for (size_t i = 0; i < scene->entities->len; i++) {
    const TS_Entity entity = g_array_index(scene->entities, TS_Entity, i);
    printf("- Entity %ld\n", entity);
  }
  return;
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
    const TS_Entity entity = g_array_index(scene->entities, TS_Entity, i);
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

void ts_print_systems(TS_Scene_t *scene) {
  printf("Active Systems %d:\n", scene->systems->len);
  for (size_t i = 0; i < scene->systems->len; i++) {
    const TS_System_Handler *system =
        g_array_index(scene->systems, TS_System_Handler *, i);
    printf("- %s (Priority: %d) (%s)\n", system->id, system->priority,
           system->plugin->path);
  }
  return;
}
