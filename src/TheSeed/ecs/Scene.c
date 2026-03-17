#include "TheSeed/ecs/Scene.h"
#include "Scene_internal.h"
#include "TheSeed/core/logging.h"
#include "TheSeed/core/utils.h"
#include <dlfcn.h>
#include <glib.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

TS_Scene *ts_create_scene() {
  TS_Scene *scene = (TS_Scene *)malloc(sizeof(TS_Scene));
  ts_assert(scene, "Malloc failed while creating a scene");
  scene->plugins = g_array_new(FALSE, FALSE, sizeof(TS_Plugin_Handler *));
  scene->entities = g_array_new(FALSE, FALSE, sizeof(TS_Entity));
  scene->entity_counter = 0;
  scene->components = g_array_new(FALSE, FALSE, sizeof(TS_Component_Handler *));
  scene->systems = g_array_new(FALSE, FALSE, sizeof(TS_System_Handler *));
  scene->should_reload = 0;
  scene->should_terminate = 0;
  return scene;
}

void ts_destroy_scene(TS_Scene *scene) {
  if (!scene) {
    return;
  }
  // Unloading all plugins
  // This will result in destroying all the components and systems with it.
  size_t plugins_len = scene->plugins->len;
  for (size_t i = 0; i < plugins_len; i++) {
    const TS_Plugin_Handler *plugin =
        g_array_index(scene->plugins, TS_Plugin_Handler *, 0);
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

TS_Entity *ts_get_entities(size_t *size, const TS_Scene *scene) {
  ts_assert_abort_value(size, NULL, "Size is NULL during ts_get_entities");
  ts_assert_abort_value(scene, NULL, "Scene is NULL during ts_get_entities");
  *size = scene->entities->len;
  TS_Entity *data = (TS_Entity *)malloc(sizeof(TS_Entity) * *size);
  memcpy(data, scene->entities->data, sizeof(TS_Entity) * *size);
  return data;
}

int ts_load_plugin(TS_Scene *scene, const char *plugin_name) {
  ts_assert_abort_value(scene, -1, "Scene is NULL during ts_load_plugin");
  long does_exist = ts_get_plugin_index(scene, plugin_name);
  if (does_exist != -1L) {
    ts_warn("Plugin `%s` already loaded", plugin_name);
    return 1;
  }
  TS_Plugin_Handler *plugin =
      (TS_Plugin_Handler *)malloc(sizeof(TS_Plugin_Handler));
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
      char *copy_id = ts_copy_char_ptr(component->id);
      ts_remove_component(scene, component->entity, copy_id);
      free(copy_id);
      i--;
    }
  }

  // Destroy the plugins
  TS_Plugin_Handler *plugin =
      g_array_index(scene->plugins, TS_Plugin_Handler *, index);
  char *plugin_name_copy = ts_copy_char_ptr(plugin->path);
  dlclose(plugin->fd);
  free(plugin->path);
  free(plugin);
  g_array_remove_index(scene->plugins, index);

  ts_debug("Unloaded Plugin: %s", plugin_name_copy);
  free(plugin_name_copy);

  return 0;
}

char **ts_get_plugins(size_t *size, const TS_Scene *scene) {
  ts_assert_abort_value(size, NULL, "Size is NULL during ts_get_plugins");
  ts_assert_abort_value(scene, NULL, "Scene is NULL during ts_get_plugins");
  *size = scene->plugins->len;
  char **data = (char **)malloc(sizeof(char *) * *size);
  for (size_t i = 0; i < *size; i++) {
    TS_Plugin_Handler *plugin =
        g_array_index(scene->plugins, TS_Plugin_Handler *, i);
    data[i] = ts_copy_char_ptr(plugin->path);
  }
  return data;
}

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

  TS_Plugin_Handler *plugin;
  TS_Component_Creator creator =
      ts_get_abi_symbol(&plugin, scene, CREATOR_FUNCION_PREFIX, component_id);
  TS_Component_Destroyer destroyer = ts_get_abi_symbol_from_plugin(
      scene, plugin, DESTROYER_FUNCION_PREFIX, component_id);
  TS_Component_Schema_Function schema_function = ts_get_abi_symbol_from_plugin(
      scene, plugin, SCHEMA_FUNCTION_PREFIX, component_id);

  if (!creator) {
    ts_error("Failed to find creator for `%s`", component_id);
    return 1;
  }
  if (!destroyer) {
    ts_error("Failed to find destroyer for `%s`", component_id);
    return 1;
  }
  if (!schema_function) {
    ts_error("Failed to find schema function for `%s`", component_id);
    return 1;
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
  component_handler->plugin = plugin;
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
  ts_assert_abort_value(scene, NULL,
                        "Scene is NULL during ts_entity_get_component");
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

  // Copy for the finish debug message the component_id
  char *copy_id = ts_copy_char_ptr(component_id);

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

  ts_debug("Removed component `%s` from entity %ld", copy_id, entity);
  free(copy_id);
  return 0;
}

char **ts_get_components_of_entity(size_t *size, const TS_Scene *scene,
                                   const TS_Entity entity_id) {
  ts_assert_abort_value(size, NULL,
                        "Size is NULL during ts_get_components_of_entity");
  ts_assert_abort_value(scene, NULL,
                        "Scene is NULL during ts_get_components_of_entity");
  ts_assert_abort_value(
      entity_id >= scene->entity_counter, NULL,
      "Entity ID is invalid during ts_get_components_of_entity");
  *size = 0;
  for (size_t i = 0; i < scene->components->len; i++) {
    TS_Component_Handler *component =
        g_array_index(scene->components, TS_Component_Handler *, i);
    if (component->entity == entity_id) {
      *size += 1;
    }
  }
  char **data = (char **)malloc(*size * sizeof(char *));
  for (size_t i = 0; i < scene->components->len; i++) {
    TS_Component_Handler *component =
        g_array_index(scene->components, TS_Component_Handler *, i);
    if (component->entity == entity_id) {
      data[i] = ts_copy_char_ptr(component->id);
    }
  }
  return data;
}

static int ts_default_selector(TS_Scene *scene, const TS_Entity entity_id) {
  return 0;
}

int ts_add_system(TS_Scene *scene, const char *system_id, int priority) {
  ts_assert_abort_value(scene, -1, "Scene is NULL during ts_add_system");
  ts_assert_abort_value(system_id, -1, "Id is NULL during ts_add_system");
  long index = ts_get_system_index(scene, system_id);

  if (index != -1L) {
    // Already exists, won't add
    return 1;
  }

  // Find the selector and system function

  // Working string
  TS_Plugin_Handler *plugin = NULL;
  TS_System_Function system =
      ts_get_abi_symbol(&plugin, scene, SYSTEM_FUNCTION_PREFIX, system_id);
  TS_System_Selector selector = ts_get_abi_symbol_from_plugin(
      scene, plugin, SYSTEM_SELECTOR_PREFIX, system_id);
  TS_System_Attacher attacher = ts_get_abi_symbol_from_plugin(
      scene, plugin, SYSTEM_ATTACH_PREFIX, system_id);
  TS_System_Detacher detacher = ts_get_abi_symbol_from_plugin(
      scene, plugin, SYSTEM_DETACH_PREFIX, system_id);
  TS_System_Groups *groups = ts_get_abi_symbol_from_plugin(
      scene, plugin, SYSTEM_GROUPS_PREFIX, system_id);

  if (!system) {
    ts_warn("Failed to find system function in the system `%s`", system_id);
    return 2;
  }
  // Set default functions
  if (!selector) {
    // ts_debug("Failed to find selector in the system `%s`", system_id);
    selector = ts_default_selector;
  }
  if (!groups) {
    ts_debug("System %s has groups not defined", system_id);
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
  system_handler->plugin = plugin;
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

char **ts_get_systems(size_t *size, const TS_Scene *scene) {
  ts_assert_abort_value(size, NULL, "Size is NULL during ts_get_plugins");
  ts_assert_abort_value(scene, NULL, "Scene is NULL during ts_get_plugins");
  *size = scene->systems->len;
  char **data = (char **)malloc(sizeof(char *) * *size);
  for (size_t i = 0; i < *size; i++) {
    TS_System_Handler *system =
        g_array_index(scene->systems, TS_System_Handler *, i);
    data[i] = ts_copy_char_ptr(system->id);
  }
  return data;
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

int ts_tick_scene(TS_Scene *scene) {
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
    TS_System_Groups groups = 0;
    if (system->groups == NULL) {
      groups = 0;
    } else {
      groups = *system->groups;
    }

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

  if (scene->should_terminate) {
    return 0;
  }
  // Check if the scene should reload
  if (scene->should_reload) {
    ts_debug("ECS system should be reloaded");
    scene->should_reload = 0; // Reset
    ts_reload_all_plugins(scene);
  }

  return 1;
}

int ts_remove_system(TS_Scene *scene, const char *system_id) {
  ts_assert_abort_value(scene, -1, "Scene is NULL during ts_remove_system");
  ts_assert_abort_value(system_id, -1, "Id is NULL during ts_remove_system");
  long index = ts_get_system_index(scene, system_id);

  if (index == -1L) {
    ts_warn("System `%s` doesn't exist", system_id);
    return 1;
  }

  TS_System_Handler *system =
      g_array_index(scene->systems, TS_System_Handler *, index);
  char *system_id_copy = ts_copy_char_ptr(system->id);

  if (system->detacher) {
    ts_debug("Running detacher for system `%s`", system->id);
    system->detacher(scene);
  }

  free(system->id);
  free(system);
  g_array_remove_index(scene->systems, index);

  ts_debug("System `%s` was removed", system_id_copy);
  free(system_id_copy);

  return 0;
}

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

  TS_Plugin_Handler *plugin =
      g_array_index(scene->plugins, TS_Plugin_Handler *, index);

  // Pre Unload operations

  GArray *components_to_reconstruct =
      g_array_new(FALSE, FALSE, sizeof(TS_Component_Serialization *));
  GArray *components_to_reconstruct_entities =
      g_array_new(FALSE, FALSE, sizeof(TS_Entity));

  for (size_t i = 0; i < scene->components->len; i++) {
    TS_Component_Handler *component =
        g_array_index(scene->components, TS_Component_Handler *, i);

    if (component->plugin == plugin) {
      // Serialize the component and remove it from the scene.
      TS_Component_Serialization *serialization =
          ts_serialize_component_internal(component);
      g_array_append_val(components_to_reconstruct, serialization);
      g_array_append_val(components_to_reconstruct_entities, component->entity);
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
    const TS_Entity reconstruction_entity =
        g_array_index(components_to_reconstruct_entities, TS_Entity, i);

    // Create and get the component handler
    status = ts_add_component(scene, reconstruction_entity,
                              reconstruction->component_name);
    if (status) {
      ts_warn("Failed to create component `%s` for entity %ld",
              reconstruction->component_name, reconstruction_entity);
      ts_destroy_component_serialization(reconstruction);
      continue;
    }
    void *component = ts_entity_get_component(scene, reconstruction_entity,
                                              reconstruction->component_name);
    ts_assert(component, "NULL component created during reload");
    TS_Component_Handler *handler =
        ts_find_handler_for_component(scene, component);
    ts_assert(handler, "NULL handler created during reload");

    // Perform the deserialization of the component
    status = ts_deserialize_component_internal(scene, reconstruction_entity,
                                               reconstruction);
    ts_debug("Component `%s` was reloaded for entity %ld",
             reconstruction->component_name, reconstruction_entity);

    // Clean up the helper array and the serialization
    ts_destroy_component_serialization(reconstruction);
  }
  g_array_free(components_to_reconstruct, TRUE);
  g_array_free(components_to_reconstruct_entities, TRUE);

  ts_debug("Reloaded Plugin `%s` with `%s`", plugin_path, new_plugin_path);

  return status;
}

int ts_reload_all_plugins(TS_Scene *scene) {
  ts_assert_abort_value(scene, -1,
                        "Scene is NULL during ts_reload_all_plugins");
  GArray *plugins = g_array_copy(scene->plugins);

  for (size_t i = 0; i < plugins->len; i++) {
    TS_Plugin_Handler *plugin = g_array_index(plugins, TS_Plugin_Handler *, i);
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
  field->type = type;
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

TS_Primitive_Type ts_get_field_type(TS_Component_Schema *schema,
                                    char *field_name) {
  ts_assert(schema, "Schema is null during ts_get_getter");
  TS_Component_Field *field = ts_get_field(schema, field_name);
  ts_assert(field, "Field `%s` not found during the ts_get_field_type",
            field_name);
  return field->type;
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
  ts_assert_abort_value(setter, 1, "No setter found for the field `%s`", field);
  setter(component, data);
  return 0;
}

TS_Component_Schema *ts_get_schema_of_component(TS_Scene *scene,
                                                void *component) {
  TS_Component_Handler *handler =
      ts_find_handler_for_component(scene, component);
  ts_assert_abort_value(handler, NULL,
                        "Handler is null during ts_get_schema_of_component");
  ts_assert_abort_value(handler->schema, NULL,
                        "Schema is null during ts_get_schema_of_component");
  return handler->schema;
}

void ts_set_scene_terminate(TS_Scene *scene) { scene->should_terminate = 1; }

static size_t ts_get_byte_length(const char *data) {
  size_t length;
  memcpy(&length, data, sizeof(size_t));
  return length;
}

char *ts_serialize_plugin(const TS_Scene *scene, const char *plugin_id) {
  ts_assert_abort_value(scene, NULL,
                        "Scene is NULL during ts_serialize_plugin");
  ts_assert_abort_value(plugin_id, NULL,
                        "Plugin ID is NULL during ts_serialize_plugin");

  long plugin_index = ts_get_plugin_index(scene, plugin_id);
  if (plugin_index == -1L) {
    return NULL;
  }

  size_t allocation = sizeof(size_t) + strlen(plugin_id) + 1;
  char *data = (char *)malloc(allocation);
  char *iter = data;

  memcpy(iter, &allocation, sizeof(size_t));
  iter += sizeof(size_t);

  memcpy(iter, plugin_id, strlen(plugin_id) + 1);

  return data;
}

char *ts_serialize_system(const TS_Scene *scene, const char *system_id) {
  ts_assert_abort_value(scene, NULL,
                        "Scene is NULL during ts_serialize_plugin");
  ts_assert_abort_value(system_id, NULL,
                        "System ID is NULL during ts_serialize_plugin");

  long system_index = ts_get_system_index(scene, system_id);
  if (system_index == -1L) {
    return NULL;
  }

  TS_System_Handler *system_handler =
      g_array_index(scene->systems, TS_System_Handler *, system_index);

  size_t allocation =
      sizeof(size_t) + strlen(system_handler->id) + 1 + sizeof(int);
  char *data = (char *)malloc(allocation);
  char *iter = data;

  memcpy(iter, &allocation, sizeof(size_t));
  iter += sizeof(size_t);

  memcpy(iter, system_handler->id, strlen(system_handler->id) + 1);
  iter += strlen(system_handler->id) + 1;

  memcpy(iter, &system_handler->priority, sizeof(int));

  return data;
}

static char *
ts_serialize_component_item(const TS_Scene *scene,
                            const TS_Component_Serialization_Item *item) {
  ts_assert_abort_value(scene, NULL,
                        "Scene is NULL during ts_serialize_component_item");
  ts_assert_abort_value(item, NULL,
                        "Item is NULL during ts_serialize_component_item");

  size_t allocation =
      sizeof(size_t) + strlen(item->field_name) + 1 + item->size;
  char *data = malloc(allocation);
  char *iter = data;

  memcpy(iter, &allocation, sizeof(size_t));
  iter += sizeof(size_t);

  memcpy(iter, item->field_name, strlen(item->field_name) + 1);
  iter += strlen(item->field_name) + 1;

  memcpy(iter, item->data, item->size);

  return data;
}

char *ts_serialize_component(const TS_Scene *scene, const void *component) {
  ts_assert_abort_value(scene, NULL,
                        "Scene is NULL during ts_serialize_component");
  ts_assert_abort_value(component, NULL,
                        "Component is NULL during ts_serialize_component");

  TS_Component_Handler *component_handler =
      ts_find_handler_for_component(scene, component);
  if (!component_handler) {
    ts_error("Failed to find the component in the register during "
             "ts_serialize_component");
    return NULL;
  }

  TS_Component_Serialization *serialization =
      ts_serialize_component_internal(component_handler);
  ts_assert(serialization, "Failed to internally serialize the component");

  size_t allocation =
      sizeof(size_t) + strlen(serialization->component_name) + 1;

  char **fields = malloc(serialization->fields->len * sizeof(char *));

  for (size_t i = 0; i < serialization->fields->len; i++) {
    TS_Component_Serialization_Item *item = g_array_index(
        serialization->fields, TS_Component_Serialization_Item *, i);
    char *serialization_item = ts_serialize_component_item(scene, item);
    size_t length_of_item = ts_get_byte_length(serialization_item);
    allocation += length_of_item;
    fields[i] = serialization_item;
  }

  char *data = malloc(allocation);
  char *iter = data;

  memcpy(iter, &allocation, sizeof(size_t));
  iter += sizeof(size_t);

  memcpy(iter, serialization->component_name,
         strlen(serialization->component_name) + 1);
  iter += strlen(serialization->component_name) + 1;

  for (size_t i = 0; i < serialization->fields->len; i++) {
    size_t offset = ts_get_byte_length(fields[i]);

    memcpy(iter, fields[i], offset);

    iter += offset;
    free(fields[i]);
  }
  free(fields);
  ts_destroy_component_serialization(serialization);

  return data;
}

char *ts_serialize_entity(const TS_Scene *scene, const TS_Entity entity_id) {
  ts_assert_abort_value(scene, NULL,
                        "Scene is NULL during ts_serialize_entity");
  ts_assert_abort_value(entity_id < scene->entity_counter, NULL,
                        "Entity ID is invalid");

  // Compute the length of the component array
  size_t component_counter = 0;
  for (size_t i = 0; i < scene->components->len; i++) {
    TS_Component_Handler *component_handler =
        g_array_index(scene->components, TS_Component_Handler *, i);
    if (component_handler->entity == entity_id) {
      component_counter++;
    }
  }

  // Create the component array
  char **component_serializations =
      (char **)malloc(sizeof(char *) * component_counter);

  size_t allocation = sizeof(size_t) + sizeof(TS_Entity);

  // Compute the component array and also compute the allocation length
  size_t component_index = 0;
  for (size_t i = 0; i < scene->components->len; i++) {
    TS_Component_Handler *component_handler =
        g_array_index(scene->components, TS_Component_Handler *, i);
    if (component_handler->entity == entity_id) {
      char *component_serialization =
          ts_serialize_component(scene, component_handler->component);
      component_serializations[component_index++] = component_serialization;

      size_t offset = ts_get_byte_length(component_serialization);
      allocation += offset;
    }
  }

  char *data = (char *)malloc(allocation);
  char *iter = data;

  memcpy(iter, &allocation, sizeof(size_t));
  iter += sizeof(size_t);

  memcpy(iter, &entity_id, sizeof(TS_Entity));
  iter += sizeof(TS_Entity);

  for (size_t i = 0; i < component_counter; i++) {
    char *component = component_serializations[i];
    size_t offset = ts_get_byte_length(component);
    memcpy(iter, component, offset);
    iter += offset;

    // Cleanup
    free(component);
  }
  free(component_serializations);

  return data;
}

char *ts_serialize_scene(const TS_Scene *scene) {
  ts_assert_abort_value(scene, NULL, "Scene is NULL during ts_serialize_scene");

  size_t allocation =
      sizeof(size_t) + sizeof(size_t) + sizeof(size_t) + sizeof(size_t);

  // Convert the lengths into proper size_t
  size_t plugin_size = (size_t)scene->plugins->len;
  size_t system_size = (size_t)scene->systems->len;
  size_t entity_size = (size_t)scene->entities->len;

  char **plugin_serializations = (char **)malloc(sizeof(char *) * plugin_size);
  char **system_serializations = (char **)malloc(sizeof(char *) * system_size);
  char **entity_serializations = (char **)malloc(sizeof(char *) * entity_size);

  // Gather all the serializations and figure out how much to allocate in the
  // end
  for (size_t i = 0; i < plugin_size; i++) {
    TS_Plugin_Handler *plugin_handler =
        g_array_index(scene->plugins, TS_Plugin_Handler *, i);
    plugin_serializations[i] = ts_serialize_plugin(scene, plugin_handler->path);
    size_t offset = ts_get_byte_length(plugin_serializations[i]);
    allocation += offset;
  }

  for (size_t i = 0; i < system_size; i++) {
    TS_System_Handler *system_handler =
        g_array_index(scene->systems, TS_System_Handler *, i);
    system_serializations[i] = ts_serialize_system(scene, system_handler->id);
    size_t offset = ts_get_byte_length(system_serializations[i]);
    allocation += offset;
  }

  for (size_t i = 0; i < entity_size; i++) {
    TS_Entity entity_id = g_array_index(scene->entities, TS_Entity, i);
    entity_serializations[i] = ts_serialize_entity(scene, entity_id);
    size_t offset = ts_get_byte_length(entity_serializations[i]);
    allocation += offset;
  }

  // Construct the final serialization
  char *data = (char *)malloc(allocation);
  char *iter = data;

  memcpy(iter, &allocation, sizeof(size_t));
  iter += sizeof(size_t);

  memcpy(iter, &plugin_size, sizeof(size_t));
  iter += sizeof(size_t);

  memcpy(iter, &system_size, sizeof(size_t));
  iter += sizeof(size_t);

  memcpy(iter, &entity_size, sizeof(size_t));
  iter += sizeof(size_t);

  for (size_t i = 0; i < plugin_size; i++) {
    size_t offset = ts_get_byte_length(plugin_serializations[i]);
    memcpy(iter, plugin_serializations[i], offset);
    iter += offset;
    free(plugin_serializations[i]);
  }

  for (size_t i = 0; i < system_size; i++) {
    size_t offset = ts_get_byte_length(system_serializations[i]);
    memcpy(iter, system_serializations[i], offset);
    iter += offset;
    free(system_serializations[i]);
  }

  for (size_t i = 0; i < entity_size; i++) {
    size_t offset = ts_get_byte_length(entity_serializations[i]);
    memcpy(iter, entity_serializations[i], offset);
    iter += offset;
    free(entity_serializations[i]);
  }

  // Cleanup
  free(plugin_serializations);
  free(system_serializations);
  free(entity_serializations);

  return data;
}

int ts_deserialize_plugin(TS_Scene *scene, const char *data) {
  ts_assert_abort_value(scene, 2, "Scene is NULL during ts_deserialize_plugin");
  ts_assert_abort_value(data, 2, "Data is NULL during ts_deserialize_plugin");

  const char *iter = data + sizeof(size_t);

  ts_load_plugin(scene, iter);

  return 0;
}

int ts_deserialize_system(TS_Scene *scene, const char *data) {
  ts_assert_abort_value(scene, 2, "Scene is NULL during ts_deserialize_plugin");
  ts_assert_abort_value(data, 2, "Data is NULL during ts_deserialize_plugin");

  size_t size = ts_get_byte_length(data);
  const char *iter = data + sizeof(size_t);
  size -= sizeof(size_t);

  const char *system_name = iter;
  const int *system_priority =
      (int *)(iter + (sizeof(char) * size - sizeof(int)));

  ts_add_system(scene, system_name, *system_priority);

  return 0;
}

static TS_Component_Serialization_Item *
ts_deserialize_component_item(TS_Scene *scene, const char *data) {
  const size_t size = ts_get_byte_length(data);
  const char *field_name = data + sizeof(size_t);
  const size_t field_name_size = ts_len_till_null(field_name, sizeof(char));
  const size_t data_size = size - sizeof(size_t) - field_name_size - 1;
  const void *field_data = data + sizeof(size_t) + field_name_size + 1;

  TS_Component_Serialization_Item *item =
      ts_create_component_serialization_item(field_name, field_data, data_size,
                                             TS_L);

  return item;
}

int ts_deserialize_component(TS_Scene *scene, const TS_Entity entity,
                             const char *data) {
  const size_t size = ts_get_byte_length(data);

  const char *component_name = data += sizeof(size_t);

  TS_Component_Serialization *serialization =
      ts_create_component_serialization(component_name);

  const char *end = data + size;
  const char *iter = data + sizeof(size_t) + strlen(component_name) + 1;

  while (iter < end) {
    size_t length = ts_get_byte_length(iter);

    TS_Component_Serialization_Item *item =
        ts_deserialize_component_item(scene, iter);
    g_array_append_val(serialization->fields, item);

    iter += length;
  }

  if (iter != end) {
    ts_error(
        "Invalid component_item deserialization while deserializing entity %ld",
        entity);
    ts_destroy_component_serialization(serialization);
    return 1;
  }

  ts_deserialize_component_internal(scene, entity, serialization);

  return 0;
}

int ts_deserialize_entity(TS_Scene *scene, const char *data) {
  const size_t size = ts_get_byte_length(data);

  const TS_Entity entity =
      ts_add_entity(scene); // We disregard the entity that was serialized

  const char *end = data + size;
  const char *iter = data + sizeof(size_t) + sizeof(TS_Entity);

  while (iter < end) {
    const size_t length = ts_get_byte_length(iter);
    ts_deserialize_component(scene, entity, iter);
    iter += length;
  }

  if (iter != end) {
    ts_error("Invalid component deserialization while deserializing entity %ld",
             entity);
    return 1;
  }

  return 0;
}

int ts_deserialize_scene(TS_Scene *scene, const char *data) { return 0; }

int ts_serialize_scene_to_file(const TS_Scene *scene, const char *path) {
  ts_assert_abort_value(scene, 1,
                        "Scene is NULL during ts_serialize_scene_to_file");
  ts_assert_abort_value(path, 1,
                        "Path is NULL during ts_serialize_scene_to_file");

  // Serialize the scene
  char *data = ts_serialize_scene(scene);
  ts_assert_abort_value(
      data, 1, "Failed to serialize scene during ts_serialize_scene_to_file");

  // Extract the size from the data
  size_t size;
  memcpy(&size, data, sizeof(size_t));

  // Open the file for binary writing
  FILE *file = fopen(path, "wb");
  if (!file) {
    ts_error("Failed to open file '%s' for writing", path);
    free(data);
    return 1;
  }

  // Write the serialized data to the file
  size_t written = fwrite(data, 1, size, file);
  if (written != size) {
    ts_error("Failed to write complete data to file '%s' (%zu/%zu bytes "
             "written)",
             path, written, size);
    int status = fclose(file);
    if (status) {
      ts_error("Also failed to close the file");
      free(data);
      return 1;
    }
    free(data);
    return 1;
  }

  // Close the file
  int status = fclose(file);
  if (status) {
    ts_warn("Failed to close the file (still the write succeeded). Proceeding");
  }

  // Cleanup
  free(data);

  ts_debug("Scene serialized to file '%s' (%zu bytes)", path, size);

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
    const TS_Plugin_Handler *plugin =
        g_array_index(scene->plugins, TS_Plugin_Handler *, i);
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
