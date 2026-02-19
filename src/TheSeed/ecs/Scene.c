#include "TheSeed/ecs/Scene.h"
#include "TheSeed/core/logging.h"
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
  TS_Component_Serializer serializer;
  TS_Component_Deserializer deserializer;
  TS_Component_Activator activator;
  void *component;
} TS_Component_Handler;

typedef struct {
  char *id;
  int priority;
  int active;
  TS_Loaded_Plugin *plugin;
  TS_System_Groups *groups;
  TS_System_Selector selector;
  TS_System_Attacher attacher;
  TS_System_Detacher detacher;
  TS_System_Function system;
} TS_System_Handler;

struct TS_Scene {
  GArray *plugins;
  TS_Entity entity_counter;
  GArray *entities;
  GArray *components;
  GArray *systems;
  int should_reload;
};

typedef struct {
  char *name;
  size_t size;
  void *data;
} TS_Serialization_Field;

struct TS_Serialization {
  char *name;
  TS_Entity entity;
  GArray *fields;
};

#define TS_FIND_SYMBOL_IN_PLUGINS(plugins, id, prefix, function_var,           \
                                  plugin_var)                                  \
  for (size_t i = 0; i < plugins->len; i++) {                                  \
    TS_Loaded_Plugin *plugin = g_array_index(plugins, TS_Loaded_Plugin *, i);  \
    ts_assert(plugin,                                                          \
              "Plugin was null while searching for symbol in plugins");        \
    GString *symbol = g_string_new(id);                                        \
    g_string_prepend(symbol, prefix);                                          \
    function_var = dlsym(plugin->fd, symbol->str);                             \
    g_string_free(symbol, TRUE);                                               \
    if (function_var) {                                                        \
      plugin_var = plugin;                                                     \
      break;                                                                   \
    }                                                                          \
  }

TS_Scene *ts_create_scene() {
  TS_Scene *p = (TS_Scene *)malloc(sizeof(TS_Scene));
  ts_assert(p, "Malloc failed while creating a scene");
  p->plugins = g_array_new(FALSE, FALSE, sizeof(TS_Loaded_Plugin *));
  p->entities = g_array_new(FALSE, FALSE, sizeof(TS_Entity));
  p->entity_counter = 0;
  p->components = g_array_new(FALSE, FALSE, sizeof(TS_Component_Handler *));
  p->systems = g_array_new(FALSE, FALSE, sizeof(TS_System_Handler *));
  p->should_reload = 0;
  return p;
}

void ts_destroy_scene(TS_Scene *scene) {
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

TS_Entity ts_add_entity(TS_Scene *scene) {
  ts_assert_abort_value(scene, -1, "Scene is NULL during entity creation");
  TS_Entity entity = scene->entity_counter;
  scene->entity_counter += 1;
  g_array_append_val(scene->entities, entity);
  ts_debug("Created Entity: %ld", entity);
  return entity;
}

static long ts_get_entity_index(const TS_Scene *scene, const TS_Entity entity) {
  ts_assert(scene, "Scene is NULL during ts_get_entity_index");
  for (size_t i = 0; i < scene->entities->len; i++) {
    const TS_Entity e = g_array_index(scene->entities, TS_Entity, i);
    if (e == entity) {
      return i;
    }
  }
  return -1;
}

int ts_remove_entity(TS_Scene *scene, const TS_Entity entity) {
  ts_assert_abort_value(scene, 1, "Scene is NULL during ts_remove_entity");
  ts_debug("Removing entity %ld", entity);
  long index = ts_get_entity_index(scene, entity);
  if (index == -1L) {
    ts_warn("The entity %ld doesn't exist", entity);
    return 1;
  }
  g_array_remove_index(scene->entities, index);

  // Cleanup the components
  for (size_t i = 0; i < scene->components->len; i++) {
    TS_Component_Handler *c =
        g_array_index(scene->components, TS_Component_Handler *, i);
    if (c->entity == entity) {
      char *copy_id = ts_copy_char_ptr(c->id);
      ts_remove_component(scene, entity, copy_id);
      free(copy_id);
      i--;
    }
  }

  return 1;
}

static long ts_get_plugin_index(const TS_Scene *scene, const char *path) {
  ts_assert(scene, "Scene is NULL during ts_get_plugin_index");
  for (size_t i = 0; i < scene->plugins->len; i++) {
    const TS_Loaded_Plugin *plugin =
        g_array_index(scene->plugins, TS_Loaded_Plugin *, i);
    if (strcmp(plugin->path, path) == 0) {
      return i;
    }
  }
  return -1;
}

int ts_load_plugin(TS_Scene *scene, const char *path) {
  ts_assert_abort_value(scene, -1, "Scene is NULL during ts_load_plugin");
  long does_exist = ts_get_plugin_index(scene, path);
  if (does_exist != -1L) {
    ts_warn("Plugin `%s` already loaded", path);
    return 1;
  }
  TS_Loaded_Plugin *plugin =
      (TS_Loaded_Plugin *)malloc(sizeof(TS_Loaded_Plugin));
  ts_assert(plugin, "Malloc failed during ts_load_plugin");

  plugin->path = ts_copy_char_ptr(path);

  plugin->fd = dlopen(path, RTLD_NOW);
  if (!plugin->fd) {
    ts_error("Failed to dlopen plugin `%s`: %s", path, dlerror());
    free(plugin->path);
    free(plugin);
    return 1;
  }
  g_array_append_val(scene->plugins, plugin);
  ts_debug("Loaded Plugin: %s", path);
  return 0;
}

int ts_unload_plugin(TS_Scene *scene, const char *path) {
  ts_assert_abort_value(scene, -1, "Scene is NULL during ts_unload_plugin");
  long index = ts_get_plugin_index(scene, path);
  if (index == -1L) {
    ts_warn("Plugin `%s` is not loaded", path);
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
      char *copy_id = ts_copy_char_ptr(component->id);
      ts_remove_component(scene, component->entity, copy_id);
      free(copy_id);
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

  ts_debug("Unloaded Plugin: %s", path);

  return 0;
}

TS_Serialization *ts_create_serialization(char *name, size_t entity) {
  ts_assert(name, "Name is empty in ts_create_serialization");
  GArray *data = g_array_new(FALSE, FALSE, sizeof(TS_Serialization));

  TS_Serialization *serialization =
      (TS_Serialization *)malloc(sizeof(TS_Serialization));
  ts_assert(serialization, "Malloc failed during ts_create_serialization");
  serialization->name = ts_copy_char_ptr(name);
  serialization->entity = entity;
  serialization->fields = data;
  return serialization;
}

void ts_destroy_serialization(TS_Serialization *serialization) {
  if (!serialization) {
    return;
  }
  for (size_t i = 0; i < serialization->fields->len; i++) {
    TS_Serialization_Field field =
        g_array_index(serialization->fields, TS_Serialization_Field, i);
    free(field.name);
    free(field.data);
  }
  g_array_free(serialization->fields, TRUE);
  free(serialization->name);
  free(serialization);
  return;
}

void *ts_get_serialization(TS_Serialization *serialization, char *name) {
  ts_assert(serialization, "Serialization is NULL during ts_get_serialization");
  for (size_t i = 0; i < serialization->fields->len; i++) {
    TS_Serialization_Field field =
        g_array_index(serialization->fields, TS_Serialization_Field, i);
    if (strcmp(name, field.name) == 0) {
      return field.data;
    }
  }
  ts_debug("Failed to find in serialization the field `%s`", name);
  return NULL;
}

int ts_set_serialization(TS_Serialization *serialization, char *name,
                         size_t size, void *data) {
  ts_assert(serialization, "Serialization is NULL during ts_set_serialization");
  for (size_t i = 0; i < serialization->fields->len; i++) {
    TS_Serialization_Field field =
        g_array_index(serialization->fields, TS_Serialization_Field, i);
    if (strcmp(name, field.name) == 0) {
      // Replace the value
      // The user is required to free the value stored in there beforehand
      ts_warn("Data was overwritten during ts_set_serialization");
      field.data = data;
      return 1;
    }
  }

  // Create new field
  TS_Serialization_Field field;
  field.name = ts_copy_char_ptr(name);
  field.size = size;
  field.data = data;
  g_array_append_val(serialization->fields, field);
  return 0;
}

int ts_add_component(TS_Scene *scene, const TS_Entity entity, const char *id,
                     TS_Serialization *serialization) {
  ts_assert_abort_value(scene, -1, "Scene is NULL during ts_add_component");
  ts_assert_abort_value(id, -1, "Id is NULL during ts_add_component");
  // Check if the entity exists
  long does_exist = ts_get_entity_index(scene, entity);
  if (does_exist == -1) {
    ts_warn("Component `%s` already exists on entity %ld", id, entity);
    return 1;
  }

  TS_Loaded_Plugin *plugin1;
  TS_Loaded_Plugin *plugin2;
  TS_Loaded_Plugin *plugin3;
  TS_Loaded_Plugin *plugin4;
  TS_Loaded_Plugin *plugin5;
  TS_Component_Creator creator;
  TS_Component_Destroyer destroyer;
  TS_Component_Serializer serializer;
  TS_Component_Deserializer deserializer;
  TS_Component_Activator activator;
  TS_FIND_SYMBOL_IN_PLUGINS(scene->plugins, id, CREATOR_FUNCION_PREFIX, creator,
                            plugin1);
  TS_FIND_SYMBOL_IN_PLUGINS(scene->plugins, id, DESTROYER_FUNCION_PREFIX,
                            destroyer, plugin2);
  TS_FIND_SYMBOL_IN_PLUGINS(scene->plugins, id, SERIALIZER_FUNCTION_PREFIX,
                            serializer, plugin3);
  TS_FIND_SYMBOL_IN_PLUGINS(scene->plugins, id, DESERIALIZER_FUNCTION_PREFIX,
                            deserializer, plugin4);
  TS_FIND_SYMBOL_IN_PLUGINS(scene->plugins, id, ACTIVATOR_FUNCTION_PREFIX,
                            activator, plugin5);

  if (!creator) {
    ts_error("Failed to find creator for `%s`", id);
    return 2;
  }
  if (!destroyer) {
    ts_error("Failed to find destroyer for `%s`", id);
    return 2;
  }
  // Note that only the creator and the destroyer are required for a component
  // to exist

  // Create the actual data container
  ts_debug("Running creator for component `%s` on entity %ld", id, entity);
  void *component = creator();
  if (deserializer) {
    if (!serialization) {
      TS_Serialization *serialization =
          ts_create_serialization(ts_copy_char_ptr(id), entity);
      ts_debug("Running deserializer for component `%s` on entity %ld with "
               "default serializer",
               id, entity);
      deserializer(component, serialization);
      ts_destroy_serialization(serialization);
    } else {
      ts_debug("Running deserializer for component `%s` on entity %ld with "
               "existing serializer",
               id, entity);
      deserializer(component, serialization);
    }
  }
  if (activator) {
    ts_debug("Running activator for component `%s` on entity %ld", id, entity);
    activator(component);
  }

  // Create the component handler object
  TS_Component_Handler *component_handler =
      (TS_Component_Handler *)malloc(sizeof(TS_Component_Handler));

  component_handler->id = ts_copy_char_ptr(id);
  component_handler->entity = entity;
  component_handler->plugin = plugin1;
  component_handler->destroyer = destroyer;
  component_handler->serializer = serializer;
  component_handler->deserializer = deserializer;
  component_handler->activator = activator;
  component_handler->component = component;

  // Add the component
  g_array_append_val(scene->components, component_handler);

  ts_debug("Component `%s` added to entity %ld", id, entity);

  return 0;
}

static long ts_get_component_index_from_entity_and_id(TS_Scene *scene,
                                                      const TS_Entity entity,
                                                      const char *id) {
  ts_assert(scene,
            "Scene is NULL during ts_get_component_index_from_entity_and_id");
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

void *ts_entity_get_component(TS_Scene *scene, const TS_Entity entity,
                              const char *id) {
  ts_assert(scene, "Scene is NULL during ts_entity_get_component");
  long index = ts_get_component_index_from_entity_and_id(scene, entity, id);
  if (index == -1L) {
    return NULL;
  }

  TS_Component_Handler *component =
      g_array_index(scene->components, TS_Component_Handler *, index);
  return component->component;
}

int ts_remove_component(TS_Scene *scene, const TS_Entity entity,
                        const char *id) {
  ts_assert(scene, "Scene is NULL during ts_remove_component");
  // There can only be one association between the entity and the component
  long index = ts_get_component_index_from_entity_and_id(scene, entity, id);
  if (index == -1L) {
    ts_warn("Component `%s` doesn't exist on entity %ld", scene, entity);
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

  ts_debug("Removed component `%s` from entity %ld", id, entity);
  return 0;
}

static long ts_get_system_index_from_id(TS_Scene *scene, const char *id) {
  for (size_t i = 0; i < scene->systems->len; i++) {
    const TS_System_Handler *system =
        g_array_index(scene->systems, TS_System_Handler *, i);
    if (strcmp(system->id, id) == 0) {
      return i;
    }
  }
  return -1L;
}

int ts_add_system(TS_Scene *scene, const char *id, int priority) {
  long index = ts_get_system_index_from_id(scene, id);
  if (index != -1L) {
    // Already exists, won't add
    return 1;
  }

  // Find the selector and system function

  // Working string
  TS_Loaded_Plugin *plugin1;
  TS_Loaded_Plugin *plugin2;
  TS_Loaded_Plugin *plugin3;
  TS_Loaded_Plugin *plugin4;
  TS_Loaded_Plugin *plugin5;
  TS_System_Selector selector;
  TS_System_Function system;
  TS_System_Attacher attacher;
  TS_System_Detacher detacher;
  TS_System_Groups *groups;

  TS_FIND_SYMBOL_IN_PLUGINS(scene->plugins, id, SYSTEM_SELECTOR_PREFIX,
                            selector, plugin1);
  TS_FIND_SYMBOL_IN_PLUGINS(scene->plugins, id, SYSTEM_FUNCTION_PREFIX, system,
                            plugin2);
  TS_FIND_SYMBOL_IN_PLUGINS(scene->plugins, id, SYSTEM_ATTACH_PREFIX, attacher,
                            plugin3);
  TS_FIND_SYMBOL_IN_PLUGINS(scene->plugins, id, SYSTEM_DETACH_PREFIX, detacher,
                            plugin4);
  TS_FIND_SYMBOL_IN_PLUGINS(scene->plugins, id, SYSTEM_GROUPS_PREFIX, groups,
                            plugin5);

  if (!selector) {
    return 2;
  }
  if (!system) {
    return 2;
  }
  if (!groups) {
    printf("System %s has groups not defined\n", id);
    return 2;
  }
  if (plugin1 != plugin2) {
    return 2;
  }

  // Found everything -> We can build the system handler
  TS_System_Handler *system_handler =
      (TS_System_Handler *)malloc(sizeof(TS_System_Handler));
  system_handler->id = ts_copy_char_ptr(id);
  system_handler->active = 1;
  system_handler->priority = priority;
  system_handler->system = system;
  system_handler->groups = groups;
  system_handler->selector = selector;
  system_handler->attacher = attacher;
  system_handler->detacher = detacher;
  system_handler->plugin = plugin1;
  g_array_append_val(scene->systems, system_handler);

  // Execute the attacher
  if (attacher) {
    attacher();
  }

  // Sort for priority
  ts_sort_systems(scene);

  return 0;
}

static GArray *ts_find_entities_with_selector_and_groups(
    TS_Scene *scene, TS_System_Selector selector, int group) {
  GArray *res = g_array_new(FALSE, FALSE, sizeof(TS_Entity));
  for (size_t i = 0; i < scene->entities->len; i++) {
    TS_Entity entity = g_array_index(scene->entities, TS_Entity, i);
    if (selector(scene, entity) == group) {
      g_array_append_val(res, entity);
    }
  }
  return res;
}

void ts_tick_scene(TS_Scene *scene) {
  for (size_t i = 0; i < scene->systems->len; i++) {
    TS_System_Handler *system =
        g_array_index(scene->systems, TS_System_Handler *, i);

    // Check if the system is active and should tick
    if (!system->active) {
      continue;
    }

    // Create helper arrays
    GArray *entity_groups = g_array_new(FALSE, FALSE, sizeof(TS_Entity *));
    GArray *entity_groups_size = g_array_new(FALSE, FALSE, sizeof(size_t));
    TS_System_Groups groups = *system->groups;

    // Create all the helper arrays
    for (TS_System_Groups i = 1; i <= groups; i++) {
      GArray *entities_array =
          ts_find_entities_with_selector_and_groups(scene, system->selector, i);
      size_t n = entities_array->len;
      TS_Entity *entities = (TS_Entity *)g_array_free(entities_array, FALSE);

      g_array_append_val(entity_groups_size, n);
      g_array_append_val(entity_groups, entities);
    }
    size_t *size_array = (size_t *)g_array_free(entity_groups_size, FALSE);
    TS_Entity **entity_array = (TS_Entity **)g_array_free(entity_groups, FALSE);

    system->system(scene, entity_array, size_array);

    free(size_array);
    for (int i = 0; i < groups; i++) {
      free(entity_array[i]);
    }
    free(entity_array);
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

void ts_sort_systems(TS_Scene *scene) {
  g_array_sort(scene->systems, ts_compare_systems_priority);
  return;
}

int ts_remove_system(TS_Scene *scene, const char *id) {
  long index = ts_get_system_index_from_id(scene, id);

  if (index == -1L) {
    return 1;
  }

  TS_System_Handler *system =
      g_array_index(scene->systems, TS_System_Handler *, index);

  if (system->detacher) {
    system->detacher();
  }

  free(system->id);
  free(system);
  g_array_remove_index(scene->systems, index);
  return 0;
}

int ts_reload_plugin(TS_Scene *scene, const char *path, const char *new_path) {
  long index = ts_get_plugin_index(scene, path);
  if (index == -1L) {
    return 1;
  }

  TS_Loaded_Plugin *plugin =
      g_array_index(scene->plugins, TS_Loaded_Plugin *, index);

  // Pre Unload operations

  GArray *components_to_reconstruct =
      g_array_new(FALSE, FALSE, sizeof(TS_Serialization *));

  for (size_t i = 0; i < scene->components->len; i++) {
    TS_Component_Handler *component =
        g_array_index(scene->components, TS_Component_Handler *, i);

    if (component->plugin == plugin) {
      // Serialize the component to reconstruct it later
      TS_Serialization *serialization =
          ts_create_serialization(component->id, component->entity);

      // Serialize the component
      if (component->serializer) {
        component->serializer(component->component, serialization);
      }
      g_array_append_val(components_to_reconstruct, serialization);

      // Delete the component
      char *copy_id = ts_copy_char_ptr(component->id);
      ts_remove_component(scene, component->entity, copy_id);
      free(copy_id);
      i--;
    }
  }

  GArray *systems_to_reconstruct = g_array_new(FALSE, FALSE, sizeof(char *));
  GArray *systems_to_reconstruct_priority =
      g_array_new(FALSE, FALSE, sizeof(int));
  for (size_t i = 0; i < scene->systems->len; i++) {
    TS_System_Handler *system =
        g_array_index(scene->systems, TS_System_Handler *, i);

    if (system->plugin == plugin) {
      char *copy_id = ts_copy_char_ptr(system->id);
      g_array_append_val(systems_to_reconstruct, copy_id);
      g_array_append_val(systems_to_reconstruct_priority, system->priority);

      // Unregistering the system (also calls the detacher)
      ts_remove_system(scene, copy_id);
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
  for (size_t i = 0; i < systems_to_reconstruct->len; i++) {
    g_assert(systems_to_reconstruct->len ==
             systems_to_reconstruct_priority->len);
    char *system_id = g_array_index(systems_to_reconstruct, char *, i);
    int system_priority =
        g_array_index(systems_to_reconstruct_priority, int, i);

    ts_add_system(scene, system_id, system_priority);

    free(system_id);
    system_id = NULL;
  }
  g_array_free(systems_to_reconstruct, TRUE);
  systems_to_reconstruct = NULL;
  g_array_free(systems_to_reconstruct_priority, TRUE);
  systems_to_reconstruct_priority = NULL;

  // Reconstruct all the components
  for (size_t i = 0; i < components_to_reconstruct->len; i++) {
    TS_Serialization *reconstruction =
        g_array_index(components_to_reconstruct, TS_Serialization *, i);

    // Silently fail if something doesn't work
    status = ts_add_component(scene, reconstruction->entity,
                              reconstruction->name, reconstruction);

    // Clean up the helper array and the serialization
    ts_destroy_serialization(reconstruction);
  }
  g_array_free(components_to_reconstruct, TRUE);

  return status;
}

int ts_reload_all_plugins(TS_Scene *scene) {
  GArray *plugins = g_array_copy(scene->plugins);

  for (size_t i = 0; i < plugins->len; i++) {
    TS_Loaded_Plugin *p = g_array_index(plugins, TS_Loaded_Plugin *, i);
    ts_reload_plugin(scene, p->path, p->path);
  }

  g_array_free(plugins, TRUE);
  return 0;
}

void ts_set_scene_reload(TS_Scene *scene) {
  scene->should_reload = 1;
  return;
}

// Debug Functions

void ts_print_entities(TS_Scene *scene) {
  printf("Active Entities %d:\n", scene->entities->len);
  for (size_t i = 0; i < scene->entities->len; i++) {
    const TS_Entity entity = g_array_index(scene->entities, TS_Entity, i);
    printf("- Entity %ld\n", entity);
  }
  return;
}

void ts_print_plugins(TS_Scene *scene) {
  printf("Loaded Plugins %d:\n", scene->plugins->len);
  for (size_t i = 0; i < scene->plugins->len; i++) {
    const TS_Loaded_Plugin *plugin =
        g_array_index(scene->plugins, TS_Loaded_Plugin *, i);
    printf("- %s\n", plugin->path);
  }
}

void ts_print_components(TS_Scene *scene) {
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

void ts_print_systems(TS_Scene *scene) {
  printf("Active Systems %d:\n", scene->systems->len);
  for (size_t i = 0; i < scene->systems->len; i++) {
    const TS_System_Handler *system =
        g_array_index(scene->systems, TS_System_Handler *, i);
    printf("- %s (Priority: %d) (%s)\n", system->id, system->priority,
           system->plugin->path);
  }
  return;
}
