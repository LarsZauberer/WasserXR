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
  TS_Component_Schema *schema;
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

typedef struct {
  char *field_name;
  void *data;
} TS_Component_Serialization_Item;

typedef struct {
  TS_Entity entity_id;
  char *component_name;
  GArray *fields;
} TS_Component_Serialization;

struct TS_Scene {
  GArray *plugins;
  TS_Entity entity_counter;
  GArray *entities;
  GArray *components;
  GArray *systems;
  int should_reload;
};

struct TS_Component_Schema {
  GArray *fields;
};

struct TS_Component_Field {
  char *field_name;
  size_t size;
  TS_Field_Permission permission;
  TS_Component_Getter getter;
  TS_Component_Setter setter;
};

#define TS_FIND_SYMBOL_IN_PLUGINS(plugins, id, prefix, function_var,           \
                                  plugin_var)                                  \
  for (size_t i = 0; i < (plugins)->len; i++) {                                \
    TS_Loaded_Plugin *plugin = g_array_index(plugins, TS_Loaded_Plugin *, i);  \
    ts_assert(plugin,                                                          \
              "Plugin was null while searching for symbol in plugins");        \
    GString *symbol = g_string_new(id);                                        \
    g_string_prepend(symbol, prefix);                                          \
    (function_var) = dlsym(plugin->fd, symbol->str);                           \
    g_string_free(symbol, TRUE);                                               \
    if (function_var) {                                                        \
      (plugin_var) = plugin;                                                   \
      break;                                                                   \
    }                                                                          \
  }

TS_Scene *ts_create_scene() {
  TS_Scene *scene = (TS_Scene *)malloc(sizeof(TS_Scene));
  ts_assert(scene, "Malloc failed while creating a scene");
  scene->plugins = g_array_new(FALSE, FALSE, sizeof(TS_Loaded_Plugin *));
  scene->entities = g_array_new(FALSE, FALSE, sizeof(TS_Entity));
  scene->entity_counter = 0;
  scene->components = g_array_new(FALSE, FALSE, sizeof(TS_Component_Handler *));
  scene->systems = g_array_new(FALSE, FALSE, sizeof(TS_System_Handler *));
  scene->should_reload = 0;
  return scene;
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
  for (long i = 0; i < scene->entities->len; i++) {
    const TS_Entity eentity = g_array_index(scene->entities, TS_Entity, i);
    if (eentity == entity) {
      ts_assert(scene, "Scene is NULL during ts_get_entity_index");
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
    TS_Component_Handler *component =
        g_array_index(scene->components, TS_Component_Handler *, i);
    if (component->entity == entity) {
      char *copy_id = ts_copy_char_ptr(component->id);
      ts_remove_component(scene, entity, copy_id);
      free(copy_id);
      i--;
    }
  }

  return 1;
}

static long ts_get_plugin_index(const TS_Scene *scene, const char *path) {
  ts_assert(scene, "Scene is NULL during ts_get_plugin_index");
  for (long i = 0; i < scene->plugins->len; i++) {
    const TS_Loaded_Plugin *plugin =
        g_array_index(scene->plugins, TS_Loaded_Plugin *, i);
    if (strcmp(plugin->path, path) == 0) {
      return i;
    }
  }
  return -1;
}

int ts_load_plugin(TS_Scene *scene, const char *plugin_name) {
  ts_assert_abort_value(scene, -1, "Scene is NULL during ts_load_plugin");
  long does_exist = ts_get_plugin_index(scene, plugin_name);
  if (does_exist != -1L) {
    ts_warn("Plugin `%s` already loaded", plugin_name);
    return 1;
  }
  TS_Loaded_Plugin *plugin =
      (TS_Loaded_Plugin *)malloc(sizeof(TS_Loaded_Plugin));
  ts_assert(plugin, "Malloc failed during ts_load_plugin");

  plugin->path = ts_copy_char_ptr(plugin_name);

  plugin->fd = dlopen(plugin_name, RTLD_NOW);
  if (!plugin->fd) {
    ts_error("Failed to dlopen plugin `%s`: %s", plugin_name, dlerror());
    free(plugin->path);
    free(plugin);
    return 1;
  }
  g_array_append_val(scene->plugins, plugin);
  ts_debug("Loaded Plugin: %s", plugin_name);
  return 0;
}

int ts_unload_plugin(TS_Scene *scene, const char *plugin_name) {
  ts_assert_abort_value(scene, -1, "Scene is NULL during ts_unload_plugin");
  long index = ts_get_plugin_index(scene, plugin_name);
  if (index == -1L) {
    ts_warn("Plugin `%s` is not loaded", plugin_name);
    return 1;
  }

  // Destroy all systems associated to the plugin
  for (size_t i = 0; i < scene->systems->len; i++) {
    TS_System_Handler *system =
        g_array_index(scene->systems, TS_System_Handler *, i);
    if (strcmp(system->plugin->path, plugin_name) == 0) {
      ts_remove_system(scene, system->id);
      i--;
    }
  }

  // Destroy all the components associated to the plugin
  for (size_t i = 0; i < scene->components->len; i++) {
    TS_Component_Handler *component =
        g_array_index(scene->components, TS_Component_Handler *, i);

    if (strcmp(component->plugin->path, plugin_name) == 0) {
      ts_remove_component(scene, component->entity, component->id);
      if (strcmp(component->plugin->path, plugin_name) == 0) {
        char *copy_id = ts_copy_char_ptr(component->id);
        ts_remove_component(scene, component->entity, copy_id);
        free(copy_id);
        i--;
      }
    }
  }

  // Destroy the plugins
  TS_Loaded_Plugin *plugin =
      g_array_index(scene->plugins, TS_Loaded_Plugin *, index);
  dlclose(plugin->fd);
  free(plugin->path);
  free(plugin);
  g_array_remove_index(scene->plugins, index);

  ts_debug("Unloaded Plugin: %s", plugin_name);

  return 0;
}

static TS_Component_Handler *ts_find_handler_for_component(TS_Scene *scene,
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

// NOLINTNEXTLINE(readability-function-cognitive-complexity)
int ts_add_component(TS_Scene *scene, const TS_Entity entity_id,
                     const char *component_id) {
  ts_assert_abort_value(scene, -1, "Scene is NULL during ts_add_component");
  ts_assert_abort_value(component_id, -1, "Id is NULL during ts_add_component");
  // Check if the entity exists
  long does_exist = ts_get_entity_index(scene, entity_id);
  if (does_exist == -1) {
    ts_warn("Component `%s` already exists on entity %ld", component_id,
            entity_id);
    return 1;
  }

  TS_Loaded_Plugin *plugin1;
  TS_Loaded_Plugin *plugin2;
  TS_Loaded_Plugin *plugin3;
  TS_Component_Creator creator;
  TS_Component_Destroyer destroyer;
  TS_Component_Schema_Function schema_function;
  TS_FIND_SYMBOL_IN_PLUGINS(scene->plugins, component_id,
                            CREATOR_FUNCION_PREFIX, creator, plugin1);
  TS_FIND_SYMBOL_IN_PLUGINS(scene->plugins, component_id,
                            DESTROYER_FUNCION_PREFIX, destroyer, plugin2);
  TS_FIND_SYMBOL_IN_PLUGINS(scene->plugins, component_id,
                            SCHEMA_FUNCTION_PREFIX, schema_function, plugin3);

  if (!creator) {
    ts_error("Failed to find creator for `%s`", component_id);
    return 2;
  }
  if (!destroyer) {
    ts_error("Failed to find destroyer for `%s`", component_id);
    return 2;
  }
  if (!schema_function) {
    ts_error("Failed to find schema function for `%s`", component_id);
    return 2;
  }
  // Note that only the creator and the destroyer are required for a
  // component to exist

  // Create the actual data container
  ts_debug("Running creator for component `%s` on entity %ld", component_id,
           entity_id);
  void *component = creator();
  ts_assert(component,
            "The component returned by the creator of the "
            "component `%s` was NULL",
            component);

  TS_Component_Schema *schema = ts_create_component_schema();
  schema_function(schema);

  // Create the component handler object
  TS_Component_Handler *component_handler =
      (TS_Component_Handler *)malloc(sizeof(TS_Component_Handler));

  component_handler->id = ts_copy_char_ptr(component_id);
  component_handler->entity = entity_id;
  component_handler->plugin = plugin1;
  component_handler->destroyer = destroyer;
  component_handler->component = component;
  component_handler->schema = schema;

  // Add the component
  g_array_append_val(scene->components, component_handler);

  ts_debug("Component `%s` added to entity %ld", component_id, entity_id);

  return 0;
}

long ts_get_component_index_from_entity_and_id(TS_Scene *scene,
                                               const TS_Entity entity,
                                               const char *component_id) {
  ts_assert(scene,
            "Scene is NULL during ts_get_component_index_from_entity_and_id");
  // Entity and id can uniquely identify a component
  for (long i = 0; i < scene->components->len; i++) {
    const TS_Component_Handler *component =
        g_array_index(scene->components, TS_Component_Handler *, i);
    if (strcmp(component->id, component_id) == 0 &&
        component->entity == entity) {
      return i;
    }
  }
  return -1L;
}

void *ts_entity_get_component(TS_Scene *scene, const TS_Entity entity,
                              const char *component_id) {
  ts_assert(scene, "Scene is NULL during ts_entity_get_component");
  long index =
      ts_get_component_index_from_entity_and_id(scene, entity, component_id);
  if (index == -1L) {
    return NULL;
  }

  TS_Component_Handler *component =
      g_array_index(scene->components, TS_Component_Handler *, index);
  return component->component;
}

int ts_remove_component(TS_Scene *scene, const TS_Entity entity,
                        const char *component_id) {
  ts_assert_abort_value(scene, -1, "Scene is NULL during ts_remove_component");
  ts_assert_abort_value(component_id, -1,
                        "Id is NULL during ts_remove_component");
  // There can only be one association between the entity and the component
  long index =
      ts_get_component_index_from_entity_and_id(scene, entity, component_id);
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
  ts_destroy_component_schema(component->schema);
  // The component pointer itself should be destroyed by the destroyer function
  free(component);

  ts_debug("Removed component `%s` from entity %ld", component_id, entity);
  return 0;
}

static long ts_get_system_index_from_id(TS_Scene *scene,
                                        const char *system_id) {
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

static int ts_default_selector(TS_Scene *scene, const TS_Entity entity_id) {
  return 0;
}

// NOLINTNEXTLINE(readability-function-cognitive-complexity)
int ts_add_system(TS_Scene *scene, const char *system_id, int priority) {
  ts_assert_abort_value(scene, -1, "Scene is NULL during ts_add_system");
  ts_assert_abort_value(system_id, -1, "Id is NULL during ts_add_system");
  long index = ts_get_system_index_from_id(scene, system_id);

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

  TS_FIND_SYMBOL_IN_PLUGINS(scene->plugins, system_id, SYSTEM_SELECTOR_PREFIX,
                            selector, plugin1);
  TS_FIND_SYMBOL_IN_PLUGINS(scene->plugins, system_id, SYSTEM_FUNCTION_PREFIX,
                            system, plugin2);
  TS_FIND_SYMBOL_IN_PLUGINS(scene->plugins, system_id, SYSTEM_ATTACH_PREFIX,
                            attacher, plugin3);
  TS_FIND_SYMBOL_IN_PLUGINS(scene->plugins, system_id, SYSTEM_DETACH_PREFIX,
                            detacher, plugin4);
  TS_FIND_SYMBOL_IN_PLUGINS(scene->plugins, system_id, SYSTEM_GROUPS_PREFIX,
                            groups, plugin5);

  if (!system) {
    ts_warn("Failed to find system function in the system `%s`", system_id);
    return 2;
  }
  // Set default functions
  if (!selector) {
    ts_debug("Failed to find selector in the system `%s`", system_id);
    selector = ts_default_selector;
  }
  if (!groups) {
    ts_debug("System %s has groups not defined", system_id);
    groups = 0;
  }
  if (plugin1 != plugin2) {
    return 2;
  }

  // Found everything -> We can build the system handler
  TS_System_Handler *system_handler =
      (TS_System_Handler *)malloc(sizeof(TS_System_Handler));
  system_handler->id = ts_copy_char_ptr(system_id);
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
    ts_debug("Running attacher for system `%s`", system_id);
    attacher(scene);
  }

  // Sort for priority
  ts_debug("Sorting Systems");
  ts_sort_systems(scene);

  ts_debug("System `%s` added with priority %d", system_id, priority);

  return 0;
}

TS_Entity *ts_find_entities_with_selector_and_groups(
    size_t *size, TS_Scene *scene, TS_System_Selector selector, int group) {
  ts_assert(scene,
            "Scene is NULL during ts_find_entities_with_selector_and_groups");
  ts_assert(selector,
            "Selector is NULL during ts_find_entities_with_selector_and_groups")
      GArray *res = g_array_new(FALSE, FALSE, sizeof(TS_Entity));
  for (size_t i = 0; i < scene->entities->len; i++) {
    TS_Entity entity = g_array_index(scene->entities, TS_Entity, i);
    if (selector(scene, entity) == group) {
      g_array_append_val(res, entity);
    }
  }
  *size = res->len;
  return (TS_Entity *)g_array_free(res, FALSE);
}

void ts_tick_scene(TS_Scene *scene) {
  ts_assert(scene,
            "Scene is NULL during ts_find_entities_with_selector_and_groups");
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
      size_t num_entities = 0;
      TS_Entity *entities = ts_find_entities_with_selector_and_groups(
          &num_entities, scene, system->selector, i);

      g_array_append_val(entity_groups_size, num_entities);
      g_array_append_val(entity_groups, entities);
    }
    // size_array has length groups
    // entity_array has length groups
    // entity_array[i] has length size_array[i]
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
    ts_debug("ECS system should be reloaded");
    scene->should_reload = 0; // Reset
    ts_reload_all_plugins(scene);
  }
}

// NOLINTNEXTLINE(bugprone-easily-swappable-parameters)
static int ts_compare_systems_priority(const gconstpointer left,
                                       const gconstpointer right) {
  const TS_System_Handler *system_a = *(const TS_System_Handler **)left;
  const TS_System_Handler *system_b = *(const TS_System_Handler **)right;

  return system_a->priority - system_b->priority;
}

void ts_sort_systems(TS_Scene *scene) {
  g_array_sort(scene->systems, ts_compare_systems_priority);
}

int ts_remove_system(TS_Scene *scene, const char *system_id) {
  ts_assert_abort_value(scene, -1, "Scene is NULL during ts_remove_system");
  ts_assert_abort_value(system_id, -1, "Id is NULL during ts_remove_system");
  long index = ts_get_system_index_from_id(scene, system_id);

  if (index == -1L) {
    ts_warn("System `%s` doesn't exist", system_id);
    return 1;
  }

  TS_System_Handler *system =
      g_array_index(scene->systems, TS_System_Handler *, index);

  if (system->detacher) {
    ts_debug("Running detacher for system `%s`", system->id);
    system->detacher(scene);
  }

  free(system->id);
  free(system);
  g_array_remove_index(scene->systems, index);

  ts_debug("System `%s` was removed", system_id);

  return 0;
}

static TS_Component_Serialization *
ts_create_component_serialization(TS_Entity entity_id, char *component_name) {
  TS_Component_Serialization *serialization =
      (TS_Component_Serialization *)malloc(sizeof(TS_Component_Serialization));
  serialization->entity_id = entity_id;
  serialization->component_name = ts_copy_char_ptr(component_name);
  serialization->fields =
      g_array_new(FALSE, FALSE, sizeof(TS_Component_Serialization_Item));
  return serialization;
}

static TS_Component_Serialization_Item *
ts_create_component_serialization_item(char *field_name, void *data,
                                       size_t size) {
  TS_Component_Serialization_Item *field =
      (TS_Component_Serialization_Item *)malloc(
          sizeof(TS_Component_Serialization_Item));
  field->field_name = ts_copy_char_ptr(field_name);

  void *data_loc = malloc(size);
  memcpy(data_loc, data, size);
  field->data = data_loc;
  return field;
}

static void
ts_destroy_component_serialization_item(TS_Component_Serialization_Item *item) {
  free(item->field_name);
  free(item->data);
  free(item);
}

static void
ts_destroy_component_serialization(TS_Component_Serialization *serialization) {
  for (size_t i = 0; i < serialization->fields->len; i++) {
    TS_Component_Serialization_Item *field = g_array_index(
        serialization->fields, TS_Component_Serialization_Item *, i);
    ts_destroy_component_serialization_item(field);
  }
  g_array_free(serialization->fields, TRUE);
  free(serialization->component_name);
  free(serialization);
}

static TS_Component_Serialization *
ts_serialize_component(TS_Component_Handler *handler) {
  TS_Component_Serialization *serialization =
      ts_create_component_serialization(handler->entity, handler->id);

  ts_assert(handler->schema,
            "Component `%s` has no schema during serialization", handler->id);
  for (size_t i = 0; i < handler->schema->fields->len; i++) {
    TS_Component_Field *field =
        g_array_index(handler->schema->fields, TS_Component_Field *, i);

    // Check the serialization bit
    if (field->permission == TS_Permission_Mask_Serialize) {
      continue;
    }

    if (!field->getter) {
      continue;
    }
    void *data = field->getter(handler->component);
    TS_Component_Serialization_Item *serialization_item =
        ts_create_component_serialization_item(field->field_name, data,
                                               field->size);
    g_array_append_val(serialization->fields, serialization_item);
  }

  return serialization;
}

static int ts_deserialize_component(TS_Scene *scene,
                                    TS_Component_Handler *handler,
                                    TS_Component_Serialization *serialization) {
  int status = 0;
  for (size_t i = 0; i < serialization->fields->len; i++) {
    TS_Component_Serialization_Item *item = g_array_index(
        serialization->fields, TS_Component_Serialization_Item *, i);
    status |= ts_set(scene, handler->component, item->field_name, item->data);
  }
  return status;
}

// NOLINTNEXTLINE(bugprone-easily-swappable-parameters)
int ts_reload_plugin(TS_Scene *scene, const char *plugin_path,
                     const char *new_plugin_path) {
  ts_assert_abort_value(scene, -1, "Scene is NULL during ts_reload_plugin");
  ts_assert_abort_value(plugin_path, -1,
                        "Path is NULL during ts_reload_plugin");
  ts_assert_abort_value(new_plugin_path, -1,
                        "New_Path is NULL during ts_reload_plugin");
  long index = ts_get_plugin_index(scene, plugin_path);
  if (index == -1L) {
    ts_warn("Plugin `%s` isn't loaded", plugin_path);
    return 1;
  }

  TS_Loaded_Plugin *plugin =
      g_array_index(scene->plugins, TS_Loaded_Plugin *, index);

  // Pre Unload operations

  GArray *components_to_reconstruct =
      g_array_new(FALSE, FALSE, sizeof(TS_Component_Serialization *));

  for (size_t i = 0; i < scene->components->len; i++) {
    TS_Component_Handler *component =
        g_array_index(scene->components, TS_Component_Handler *, i);

    if (component->plugin == plugin) {
      TS_Component_Serialization *serialization =
          ts_serialize_component(component);
      g_array_append_val(components_to_reconstruct, serialization);
      ts_remove_component(scene, component->entity, component->id);
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

  // Copy the path and the new_path over since it might be in a
  // systems memory location
  char *new_plugin_path_copy = ts_copy_char_ptr(new_plugin_path);
  free(plugin->path);
  plugin->path = new_plugin_path_copy;

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
    ts_assert(systems_to_reconstruct->len ==
                  systems_to_reconstruct_priority->len,
              "Reconstruction arrays are not the same length");
    char *system_id = g_array_index(systems_to_reconstruct, char *, i);
    int system_priority =
        g_array_index(systems_to_reconstruct_priority, int, i);

    ts_add_system(scene, system_id, system_priority);
    ts_debug("System `%s` was reloaded", system_id);

    free(system_id);
    system_id = NULL;
  }
  g_array_free(systems_to_reconstruct, TRUE);
  systems_to_reconstruct = NULL;
  g_array_free(systems_to_reconstruct_priority, TRUE);
  systems_to_reconstruct_priority = NULL;

  // Reconstruct all the components
  for (size_t i = 0; i < components_to_reconstruct->len; i++) {
    TS_Component_Serialization *reconstruction = g_array_index(
        components_to_reconstruct, TS_Component_Serialization *, i);

    // Silently fail if something doesn't work
    status = ts_add_component(scene, reconstruction->entity_id,
                              reconstruction->component_name);
    void *component = ts_entity_get_component(scene, reconstruction->entity_id,
                                              reconstruction->component_name);
    ts_assert(component, "NULL component created during reload");
    status = ts_deserialize_component(scene, component, reconstruction);
    ts_debug("Component `%s` was reloaded for entity %ld",
             reconstruction->component_name, reconstruction->entity_id);

    // Clean up the helper array and the serialization
    ts_destroy_component_serialization(reconstruction);
  }
  g_array_free(components_to_reconstruct, TRUE);

  ts_debug("Reloaded Plugin `%s` with `%s`", plugin_path, new_plugin_path);

  return status;
}

int ts_reload_all_plugins(TS_Scene *scene) {
  ts_assert_abort_value(scene, -1,
                        "Scene is NULL during ts_reload_all_plugins");
  GArray *plugins = g_array_copy(scene->plugins);

  for (size_t i = 0; i < plugins->len; i++) {
    TS_Loaded_Plugin *plugin = g_array_index(plugins, TS_Loaded_Plugin *, i);
    char *path_before = ts_copy_char_ptr(plugin->path);
    char *path_after = ts_copy_char_ptr(plugin->path);
    ts_reload_plugin(scene, path_before, path_after);
    free(path_before);
    free(path_after);
  }

  g_array_free(plugins, TRUE);

  ts_debug("Full reload of all plugins finished");

  return 0;
}

void ts_set_scene_reload(TS_Scene *scene) {
  ts_assert(scene, "Scene is NULL during ts_set_scene_reload");
  scene->should_reload = 1;
}

TS_Component_Schema *ts_create_component_schema() {
  TS_Component_Schema *schema =
      (TS_Component_Schema *)malloc(sizeof(TS_Component_Schema));
  GArray *fields_array =
      g_array_new(FALSE, FALSE, sizeof(TS_Component_Field *));
  schema->fields = fields_array;
  return schema;
}

// // NOLINTNEXTLINE(bugprone-easily-swappable-parameters)
TS_Component_Field *ts_create_component_field(char *field_name, size_t size,
                                              TS_Primitive_Type type,
                                              TS_Field_Permission permission,
                                              TS_Component_Getter getter,
                                              TS_Component_Setter setter) {
  TS_Component_Field *field =
      (TS_Component_Field *)malloc(sizeof(TS_Component_Field));

  field->field_name = ts_copy_char_ptr(field_name);
  field->size = size;
  field->permission = permission;
  field->getter = getter;
  field->setter = setter;

  return field;
}

void ts_destroy_component_schema(TS_Component_Schema *schema) {
  if (!schema) {
    return;
  }
  ts_assert(schema->fields,
            "The fields in the schema are NULL during schema destruction");
  for (size_t i = 0; i < schema->fields->len; i++) {
    TS_Component_Field *field =
        g_array_index(schema->fields, TS_Component_Field *, i);
    ts_destroy_component_field(field);
  }
  g_array_free(schema->fields, TRUE);
  free(schema);
}

void ts_destroy_component_field(TS_Component_Field *field) {
  free(field->field_name);
  free(field);
}

int ts_add_field_to_component_schema(TS_Component_Schema *schema,
                                     TS_Component_Field *field) {
  for (size_t i = 0; i < schema->fields->len; i++) {
    TS_Component_Field *other =
        g_array_index(schema->fields, TS_Component_Field *, i);
    ts_assert_abort_value(field != other, 1,
                          "Schema field has been added twice");
    ts_assert_abort_value(strcmp(field->field_name, other->field_name) != 0, 1,
                          "Schema field has been added twice")
  }
  g_array_append_val(schema->fields, field);
  return 0;
}

TS_Component_Field *ts_get_field(TS_Component_Schema *schema,
                                 char *field_name) {
  ts_assert_abort_value(schema, NULL, "Schema is null during ts_get_field");
  for (size_t i = 0; i < schema->fields->len; i++) {
    TS_Component_Field *field =
        g_array_index(schema->fields, TS_Component_Field *, i);
    if (strcmp(field->field_name, field_name) == 0) {
      return field;
    }
  }
  return NULL;
}

TS_Component_Getter ts_get_field_getter(TS_Component_Schema *schema,
                                        char *field_name) {
  ts_assert_abort_value(schema, NULL, "Schema is null during ts_get_getter");
  TS_Component_Field *field = ts_get_field(schema, field_name);
  if (!field) {
    return NULL;
  }
  return field->getter;
}

TS_Component_Setter ts_get_field_setter(TS_Component_Schema *schema,
                                        char *field_name) {
  ts_assert_abort_value(schema, NULL, "Schema is null during ts_get_getter");
  TS_Component_Field *field = ts_get_field(schema, field_name);
  if (!field) {
    return NULL;
  }
  return field->setter;
}

TS_Field_Permission ts_get_field_permission(TS_Component_Schema *schema,
                                            char *field_name) {
  ts_assert(schema, "Schema is null during ts_get_getter");
  TS_Component_Field *field = ts_get_field(schema, field_name);
  ts_assert(field, "Field `%s` not found during the ts_get_field_permission",
            field_name);
  return field->permission;
}

size_t ts_get_field_size(TS_Component_Schema *schema, char *field_name) {
  ts_assert(schema, "Schema is null during ts_get_getter");
  TS_Component_Field *field = ts_get_field(schema, field_name);
  ts_assert(field, "Field `%s` not found during the ts_get_field_size",
            field_name);
  return field->size;
}

void *ts_get(TS_Scene *scene, void *component, char *field) {
  ts_assert_abort_value(scene, NULL, "Scene is null during ts_get");
  TS_Component_Handler *handler =
      ts_find_handler_for_component(scene, component);
  ts_assert_abort_value(handler, NULL,
                        "The component pointer couldn't be found in the scene");

  TS_Component_Getter getter = ts_get_field_getter(handler->schema, field);
  ts_assert_abort_value(getter, NULL, "No getter found for the field `%s`",
                        field);
  return getter(component);
}

int ts_set(TS_Scene *scene, void *component, char *field, void *data) {
  ts_assert_abort_value(scene, 1, "Scene is null during ts_get");
  TS_Component_Handler *handler =
      ts_find_handler_for_component(scene, component);
  ts_assert_abort_value(handler, 1,
                        "The component pointer couldn't be found in the scene");

  TS_Component_Setter setter = ts_get_field_setter(handler->schema, field);
  ts_assert_abort_value(setter, 1, "No getter found for the field `%s`", field);
  setter(component, data);
  return 0;
}

// Debug Functions

void ts_print_entities(TS_Scene *scene) {
  printf("Active Entities %d:\n", scene->entities->len);
  for (size_t i = 0; i < scene->entities->len; i++) {
    const TS_Entity entity = g_array_index(scene->entities, TS_Entity, i);
    printf("- Entity %ld\n", entity);
  }
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
}
