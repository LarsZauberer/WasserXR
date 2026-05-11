#include "WasserXR/ecs/Scene.h"
#include "Scene_internal.h"
#include "WasserXR/ecs/logging.h"
#include "WasserXR/ecs/utils.h"
#include <dlfcn.h>
#include <glib.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

WXR_Scene *wxr_create_scene() {
  WXR_Scene *scene = (WXR_Scene *)malloc(sizeof(WXR_Scene));
  wxr_assert(scene, "Malloc failed while creating a scene");
  scene->plugins = g_array_new(FALSE, FALSE, sizeof(WXR_Plugin_Handler *));
  scene->entities = g_array_new(FALSE, FALSE, sizeof(WXR_Entity));
  scene->entity_counter = 0;
  scene->components =
      g_array_new(FALSE, FALSE, sizeof(WXR_Component_Handler *));
  scene->systems = g_array_new(FALSE, FALSE, sizeof(WXR_System_Handler *));
  scene->should_reload = 0;
  scene->should_terminate = 0;
  scene->should_load = NULL;
  return scene;
}

void wxr_destroy_scene(WXR_Scene *scene) {
  if (!scene) {
    return;
  }

  // This will unload everything from the scene
  wxr_reset_scene(scene);

  // This will result in destroying all the components and systems with it.
  const size_t plugins_len = scene->plugins->len;
  for (size_t i = 0; i < plugins_len; i++) {
    const WXR_Plugin_Handler *plugin =
        g_array_index(scene->plugins, WXR_Plugin_Handler *, 0);
    wxr_unload_plugin(scene, plugin->path);
  }

  g_array_free(scene->plugins, TRUE);
  g_array_free(scene->entities, TRUE);
  g_array_free(scene->components, TRUE);
  g_array_free(scene->systems, TRUE);

  free(scene);
}

void wxr_reset_scene(WXR_Scene *scene) {
  wxr_assert_abort(scene, "Scene is NULL during wxr_reset_scene");
  // Clean up all the systems
  const size_t systems_len = scene->systems->len;
  for (size_t i = 0; i < systems_len; i++) {
    const WXR_System_Handler *system =
        g_array_index(scene->systems, WXR_System_Handler *, 0);
    wxr_remove_system(scene, system->id);
  }
  // Clean up all the rest of the entities
  const size_t entities_len = scene->entities->len;
  for (size_t i = 0; i < entities_len; i++) {
    const WXR_Entity entity = g_array_index(scene->entities, WXR_Entity, 0);
    // This will also destroy all the components associated with the entity
    wxr_remove_entity(scene, entity);
  }
  // Reset the entity counter
  scene->entity_counter = 0;
}

WXR_Entity wxr_add_entity(WXR_Scene *scene) {
  wxr_assert_abort_value(scene, 1, "Scene is NULL during entity creation");
  WXR_Entity entity = scene->entity_counter;
  scene->entity_counter += 1;
  g_array_append_val(scene->entities, entity);
  wxr_debug("Created Entity: %ld", entity);
  return entity;
}

int wxr_remove_entity(WXR_Scene *scene, const WXR_Entity entity) {
  wxr_assert_abort_value(scene, 1, "Scene is NULL during wxr_remove_entity");
  wxr_debug("Removing entity %ld", entity);
  long index = wxr_get_entity_index(scene, entity);
  if (index == -1L) {
    wxr_warn("The entity %ld doesn't exist", entity);
    return 1;
  }
  g_array_remove_index(scene->entities, index);

  // Cleanup the components
  for (size_t i = 0; i < scene->components->len; i++) {
    WXR_Component_Handler *component =
        g_array_index(scene->components, WXR_Component_Handler *, i);
    if (component->entity == entity) {
      char *copy_id = wxr_copy_char_ptr(component->id);
      wxr_remove_component(scene, entity, copy_id);
      free(copy_id);
      i--;
    }
  }

  return 0;
}

WXR_Entity *wxr_get_entities(size_t *size, const WXR_Scene *scene) {
  wxr_assert_abort_value(size, NULL, "Size is NULL during wxr_get_entities");
  wxr_assert_abort_value(scene, NULL, "Scene is NULL during wxr_get_entities");

  *size = scene->entities->len;
  if (*size == 0) {
    return NULL;
  }

  WXR_Entity *data = (WXR_Entity *)malloc(sizeof(WXR_Entity) * *size);
  memcpy(data, scene->entities->data, sizeof(WXR_Entity) * *size);
  return data;
}

int wxr_load_plugin(WXR_Scene *scene, const char *plugin_name) {
  wxr_assert_abort_value(scene, 1, "Scene is NULL during wxr_load_plugin");

  // check if empty (empty will be accepted for some reason by dlopen)
  if (strcmp(plugin_name, "") == 0) {
    return 1;
  }

  long does_exist = wxr_get_plugin_index(scene, plugin_name);
  if (does_exist != -1L) {
    wxr_warn("Plugin `%s` already loaded", plugin_name);
    return 1;
  }

  WXR_Plugin_Handler *plugin =
      (WXR_Plugin_Handler *)malloc(sizeof(WXR_Plugin_Handler));
  wxr_assert(plugin, "Malloc failed during wxr_load_plugin");

  plugin->path = wxr_copy_char_ptr(plugin_name);

  plugin->fd = dlopen(plugin_name, RTLD_NOW);
  if (!plugin->fd) {
    wxr_error("Failed to dlopen plugin `%s`: %s", plugin_name, dlerror());
    free(plugin->path);
    free(plugin);
    return 1;
  }
  g_array_append_val(scene->plugins, plugin);
  wxr_debug("Loaded Plugin: %s", plugin_name);
  return 0;
}

int wxr_unload_plugin(WXR_Scene *scene, const char *plugin_name) {
  wxr_assert_abort_value(scene, 1, "Scene is NULL during wxr_unload_plugin");
  long index = wxr_get_plugin_index(scene, plugin_name);
  if (index == -1L) {
    wxr_warn("Plugin `%s` is not loaded", plugin_name);
    return 1;
  }

  // Destroy all systems associated to the plugin
  for (size_t i = 0; i < scene->systems->len; i++) {
    WXR_System_Handler *system =
        g_array_index(scene->systems, WXR_System_Handler *, i);
    if (strcmp(system->plugin->path, plugin_name) == 0) {
      wxr_remove_system(scene, system->id);
      i--;
    }
  }

  // Destroy all the components associated to the plugin
  for (size_t i = 0; i < scene->components->len; i++) {
    WXR_Component_Handler *component =
        g_array_index(scene->components, WXR_Component_Handler *, i);

    if (strcmp(component->plugin->path, plugin_name) == 0) {
      char *copy_id = wxr_copy_char_ptr(component->id);
      wxr_remove_component(scene, component->entity, copy_id);
      free(copy_id);
      i--;
    }
  }

  // Destroy the plugins
  WXR_Plugin_Handler *plugin =
      g_array_index(scene->plugins, WXR_Plugin_Handler *, index);
  char *plugin_name_copy = wxr_copy_char_ptr(plugin->path);
  dlclose(plugin->fd);
  free(plugin->path);
  free(plugin);
  g_array_remove_index(scene->plugins, index);

  wxr_debug("Unloaded Plugin: %s", plugin_name_copy);
  free(plugin_name_copy);

  return 0;
}

char **wxr_get_plugins(size_t *size, const WXR_Scene *scene) {
  wxr_assert_abort_value(size, NULL, "Size is NULL during wxr_get_plugins");
  wxr_assert_abort_value(scene, NULL, "Scene is NULL during wxr_get_plugins");
  *size = scene->plugins->len;
  if (*size == 0) {
    return NULL;
  }
  char **data = (char **)malloc(sizeof(char *) * *size);
  for (size_t i = 0; i < *size; i++) {
    WXR_Plugin_Handler *plugin =
        g_array_index(scene->plugins, WXR_Plugin_Handler *, i);
    data[i] = wxr_copy_char_ptr(plugin->path);
  }
  return data;
}

void *wxr_add_component(WXR_Scene *scene, const WXR_Entity entity_id,
                        const char *component_id) {
  wxr_assert_abort_value(scene, NULL, "Scene is NULL during wxr_add_component");
  wxr_assert_abort_value(component_id, NULL,
                         "Id is NULL during wxr_add_component");
  // Check if the entity exists
  long entity_index = wxr_get_entity_index(scene, entity_id);
  if (entity_index == -1) {
    wxr_warn("Entity %ld doesn't exist", entity_id);
    return NULL;
  }
  void *component = wxr_entity_get_component(scene, entity_id, component_id);
  if (component) {
    wxr_warn("Component `%s` already exists on entity %ld", component_id,
             entity_id);
    return NULL;
  }

  WXR_Plugin_Handler *plugin;
  WXR_Component_Creator creator =
      wxr_get_abi_symbol(&plugin, scene, CREATOR_FUNCTION_PREFIX, component_id);
  WXR_Component_Destroyer destroyer = wxr_get_abi_symbol_from_plugin(
      scene, plugin, DESTROYER_FUNCTION_PREFIX, component_id);
  WXR_Component_Schema_Function schema_function =
      wxr_get_abi_symbol_from_plugin(scene, plugin, SCHEMA_FUNCTION_PREFIX,
                                     component_id);

  if (!creator) {
    wxr_error("Failed to find creator for `%s`", component_id);
    return NULL;
  }
  if (!destroyer) {
    wxr_error("Failed to find destroyer for `%s`", component_id);
    return NULL;
  }
  if (!schema_function) {
    wxr_error("Failed to find schema function for `%s`", component_id);
    return NULL;
  }
  // Note that only the creator and the destroyer are required for a
  // component to exist

  // Create the actual data container
  wxr_debug("Running creator for component `%s` on entity %ld", component_id,
            entity_id);
  component = creator();
  wxr_assert(component,
             "The component returned by the creator of the "
             "component `%s` was NULL",
             component);

  WXR_Component_Schema *schema = wxr_create_component_schema();
  schema_function(schema);

  // Create the component handler object
  WXR_Component_Handler *component_handler =
      (WXR_Component_Handler *)malloc(sizeof(WXR_Component_Handler));

  component_handler->id = wxr_copy_char_ptr(component_id);
  component_handler->entity = entity_id;
  component_handler->plugin = plugin;
  component_handler->destroyer = destroyer;
  component_handler->component = component;
  component_handler->schema = schema;

  // Add the component
  g_array_append_val(scene->components, component_handler);

  wxr_debug("Component `%s` added to entity %ld", component_id, entity_id);

  return component;
}

long wxr_get_component_index_from_entity_and_id(const WXR_Scene *scene,
                                                const WXR_Entity entity,
                                                const char *component_id) {
  wxr_assert(scene,
             "Scene is NULL during wxr_get_component_index_from_entity_and_id");
  // Entity and id can uniquely identify a component
  for (long i = 0; i < scene->components->len; i++) {
    const WXR_Component_Handler *component =
        g_array_index(scene->components, WXR_Component_Handler *, i);
    if (strcmp(component->id, component_id) == 0 &&
        component->entity == entity) {
      return i;
    }
  }
  return -1L;
}

void *wxr_entity_get_component(const WXR_Scene *scene, const WXR_Entity entity,
                               const char *component_id) {
  wxr_assert_abort_value(scene, NULL,
                         "Scene is NULL during wxr_entity_get_component");
  long index =
      wxr_get_component_index_from_entity_and_id(scene, entity, component_id);
  if (index == -1L) {
    return NULL;
  }

  WXR_Component_Handler *component =
      g_array_index(scene->components, WXR_Component_Handler *, index);
  return component->component;
}

int wxr_remove_component(WXR_Scene *scene, const WXR_Entity entity,
                         const char *component_id) {
  wxr_assert_abort_value(scene, 1, "Scene is NULL during wxr_remove_component");
  wxr_assert_abort_value(component_id, 1,
                         "Id is NULL during wxr_remove_component");
  // There can only be one association between the entity and the component
  long index =
      wxr_get_component_index_from_entity_and_id(scene, entity, component_id);
  if (index == -1L) {
    wxr_warn("Component `%s` doesn't exist on entity %ld", component_id,
             entity);
    return 1;
  }

  // Copy for the finish debug message the component_id
  char *copy_id = wxr_copy_char_ptr(component_id);

  WXR_Component_Handler *component =
      g_array_index(scene->components, WXR_Component_Handler *, index);
  // This is the one to remove
  g_array_remove_index(scene->components, index);
  free(component->id);
  component->destroyer(
      component->component); // Call the destroyer for the component
  wxr_destroy_component_schema(component->schema);
  // The component pointer itself should be destroyed by the destroyer function
  free(component);

  wxr_debug("Removed component `%s` from entity %ld", copy_id, entity);
  free(copy_id);
  return 0;
}

char **wxr_get_components_of_entity(size_t *size, const WXR_Scene *scene,
                                    const WXR_Entity entity_id) {
  wxr_assert_abort_value(size, NULL,
                         "Size is NULL during wxr_get_components_of_entity");
  wxr_assert_abort_value(scene, NULL,
                         "Scene is NULL during wxr_get_components_of_entity");
  wxr_assert_abort_value(
      entity_id < scene->entity_counter, NULL,
      "Entity ID is invalid during wxr_get_components_of_entity");
  *size = 0;
  for (size_t i = 0; i < scene->components->len; i++) {
    WXR_Component_Handler *component =
        g_array_index(scene->components, WXR_Component_Handler *, i);
    if (component->entity == entity_id) {
      *size += 1;
    }
  }
  if (*size == 0) {
    return NULL;
  }
  char **data = (char **)malloc(*size * sizeof(char *));
  size_t data_index = 0;
  for (size_t i = 0; i < scene->components->len; i++) {
    WXR_Component_Handler *component =
        g_array_index(scene->components, WXR_Component_Handler *, i);
    if (component->entity == entity_id) {
      data[data_index++] = wxr_copy_char_ptr(component->id);
    }
  }
  return data;
}

static int wxr_default_selector(const WXR_Scene *scene,
                                const WXR_Entity entity_id) {
  return 0;
}

int wxr_add_system(WXR_Scene *scene, const char *system_id, int priority) {
  wxr_assert_abort_value(scene, 1, "Scene is NULL during wxr_add_system");
  wxr_assert_abort_value(system_id, 1, "Id is NULL during wxr_add_system");
  long index = wxr_get_system_index(scene, system_id);

  if (index != -1L) {
    // Already exists, won't add
    return 1;
  }

  // Find the selector and system function

  // Working string
  WXR_Plugin_Handler *plugin = NULL;
  WXR_System_Function system =
      wxr_get_abi_symbol(&plugin, scene, SYSTEM_FUNCTION_PREFIX, system_id);
  WXR_System_Selector selector = wxr_get_abi_symbol_from_plugin(
      scene, plugin, SYSTEM_SELECTOR_PREFIX, system_id);
  WXR_System_Attacher attacher = wxr_get_abi_symbol_from_plugin(
      scene, plugin, SYSTEM_ATTACH_PREFIX, system_id);
  WXR_System_Detacher detacher = wxr_get_abi_symbol_from_plugin(
      scene, plugin, SYSTEM_DETACH_PREFIX, system_id);
  WXR_System_Groups *groups = wxr_get_abi_symbol_from_plugin(
      scene, plugin, SYSTEM_GROUPS_PREFIX, system_id);

  if (!system) {
    wxr_warn("Failed to find system function in the system `%s`", system_id);
    return 1;
  }
  // Set default functions
  if (!selector) {
    // wxr_debug("Failed to find selector in the system `%s`", system_id);
    selector = wxr_default_selector;
  }
  if (!groups) {
    wxr_debug("System %s has groups not defined", system_id);
  }

  // Found everything -> We can build the system handler
  WXR_System_Handler *system_handler =
      (WXR_System_Handler *)malloc(sizeof(WXR_System_Handler));
  system_handler->id = wxr_copy_char_ptr(system_id);
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
    wxr_debug("Running attacher for system `%s`", system_id);
    attacher(scene);
  }

  // Sort for priority
  wxr_debug("Sorting Systems");
  wxr_sort_systems(scene);

  wxr_debug("System `%s` added with priority %d", system_id, priority);

  return 0;
}

char **wxr_get_systems(size_t *size, const WXR_Scene *scene) {
  wxr_assert_abort_value(size, NULL, "Size is NULL during wxr_get_plugins");
  wxr_assert_abort_value(scene, NULL, "Scene is NULL during wxr_get_plugins");
  *size = scene->systems->len;
  if (*size == 0) {
    return NULL;
  }
  char **data = (char **)malloc(sizeof(char *) * *size);
  for (size_t i = 0; i < *size; i++) {
    WXR_System_Handler *system =
        g_array_index(scene->systems, WXR_System_Handler *, i);
    data[i] = wxr_copy_char_ptr(system->id);
  }
  return data;
}

WXR_Entity *wxr_find_entities_with_selector_and_groups(
    size_t *size, WXR_Scene *scene, WXR_System_Selector selector, int group) {
  wxr_assert(scene,
             "Scene is NULL during wxr_find_entities_with_selector_and_groups");
  wxr_assert(
      selector,
      "Selector is NULL during wxr_find_entities_with_selector_and_groups");
  GArray *res = g_array_new(FALSE, FALSE, sizeof(WXR_Entity));
  for (size_t i = 0; i < scene->entities->len; i++) {
    WXR_Entity entity = g_array_index(scene->entities, WXR_Entity, i);
    if (selector(scene, entity) == group) {
      g_array_append_val(res, entity);
    }
  }
  *size = res->len;
  return (WXR_Entity *)g_array_free(res, FALSE);
}

static int wxr_deserialize_scene_from_file_internal(WXR_Scene *scene,
                                                    char *path) {
  wxr_assert_abort_value(
      scene, 1, "Scene is NULL during wxr_deserialize_scene_from_file");
  wxr_assert_abort_value(path, 1,
                         "Path is NULL during wxr_deserialize_scene_from_file");

  // Open the file for binary reading
  FILE *file = fopen(path, "rb");
  if (!file) {
    wxr_error("Failed to open file '%s' for reading", path);
    return 1;
  }

  // Determine file size
  if (fseek(file, 0, SEEK_END) != 0) {
    wxr_error("Failed to seek to end of file '%s'", path);
    int status = fclose(file);
    wxr_assert_abort_value(
        !status, 1,
        "Failed to close the file during deserialization. Continuing");
    return 1;
  }

  long file_size = ftell(file);
  if (file_size < 0) {
    wxr_error("Failed to determine size of file '%s'", path);
    int status = fclose(file);
    wxr_assert_abort_value(
        !status, 1,
        "Failed to close the file during deserialization. Continuing");
    return 1;
  }

  if (fseek(file, 0, SEEK_SET) != 0) {
    wxr_error("Failed to seek to beginning of file '%s'", path);
    int status = fclose(file);
    wxr_assert_abort_value(
        !status, 1,
        "Failed to close the file during deserialization. Continuing");
    return 1;
  }

  // Allocate buffer to hold file contents
  char *data = (char *)malloc((size_t)file_size);
  if (!data) {
    wxr_error("Failed to allocate memory for file '%s' (%ld bytes)", path,
              file_size);
    int status = fclose(file);
    wxr_assert_abort_value(
        !status, 1,
        "Failed to close the file during deserialization. Continuing");
    return 1;
  }

  // Read the file data
  size_t bytes_read = fread(data, 1, (size_t)file_size, file);
  if (bytes_read != (size_t)file_size) {
    wxr_error(
        "Failed to read complete data from file '%s' (%zu/%ld bytes read)",
        path, bytes_read, file_size);
    free(data);
    int status = fclose(file);
    wxr_assert_abort_value(
        !status, 1,
        "Failed to close the file during deserialization. Continuing");
    return 1;
  }

  // Close the file
  int status = fclose(file);
  if (status) {
    wxr_warn("Failed to close the file (still the read succeeded). Proceeding");
  }

  // Reset the scene
  wxr_reset_scene(scene);

  // Deserialize the scene
  status = wxr_deserialize_scene(scene, data);
  if (status) {
    wxr_error("Failed to deserialize scene from file '%s'", path);
    free(data);
    return 1;
  }

  // Cleanup
  free(data);

  wxr_debug("Scene deserialized from file '%s' (%zu bytes)", path, bytes_read);

  return 0;
}

int wxr_tick_scene(WXR_Scene *scene) {
  wxr_assert(scene, "Scene is NULL during wxr_tick_scene");
  for (size_t i = 0; i < scene->systems->len; i++) {
    WXR_System_Handler *system =
        g_array_index(scene->systems, WXR_System_Handler *, i);

    // Check if the system is active and should tick
    if (!system->active) {
      continue;
    }

    // Create helper arrays
    GArray *entity_groups = g_array_new(FALSE, FALSE, sizeof(WXR_Entity *));
    GArray *entity_groups_size = g_array_new(FALSE, FALSE, sizeof(size_t));
    WXR_System_Groups groups = 0;
    if (system->groups == NULL) {
      groups = 0;
    } else {
      groups = *system->groups;
    }

    // Create all the helper arrays
    for (WXR_System_Groups i = 0; i < groups; i++) {
      size_t num_entities = 0;
      WXR_Entity *entities = wxr_find_entities_with_selector_and_groups(
          &num_entities, scene, system->selector, i);

      g_array_append_val(entity_groups_size, num_entities);
      g_array_append_val(entity_groups, entities);
    }
    // size_array has length groups
    // entity_array has length groups
    // entity_array[i] has length size_array[i]
    size_t *size_array = (size_t *)g_array_free(entity_groups_size, FALSE);
    WXR_Entity **entity_array =
        (WXR_Entity **)g_array_free(entity_groups, FALSE);

    // Call the system
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
  if (scene->should_load) {
    wxr_deserialize_scene_from_file_internal(scene, scene->should_load);
    free(scene->should_load);
    scene->should_load = NULL;
  }
  // Check if the scene should reload
  if (scene->should_reload) {
    wxr_debug("ECS system should be reloaded");
    scene->should_reload = 0; // Reset
    wxr_reload(scene);
  }

  return 1;
}

int wxr_remove_system(WXR_Scene *scene, const char *system_id) {
  wxr_assert_abort_value(scene, 1, "Scene is NULL during wxr_remove_system");
  wxr_assert_abort_value(system_id, 1, "Id is NULL during wxr_remove_system");
  long index = wxr_get_system_index(scene, system_id);

  if (index == -1L) {
    wxr_warn("System `%s` doesn't exist", system_id);
    return 1;
  }

  WXR_System_Handler *system =
      g_array_index(scene->systems, WXR_System_Handler *, index);
  char *system_id_copy = wxr_copy_char_ptr(system->id);

  if (system->detacher) {
    wxr_debug("Running detacher for system `%s`", system->id);
    system->detacher(scene);
  }

  free(system->id);
  free(system);
  g_array_remove_index(scene->systems, index);

  wxr_debug("System `%s` was removed", system_id_copy);
  free(system_id_copy);

  return 0;
}

int wxr_reload(WXR_Scene *scene) {
  wxr_assert_abort_value(scene, 1,
                         "Scene is NULL during ts_reload_all_plugins");

  char *serialization = wxr_serialize_scene(scene);
  wxr_reset_scene(scene);
  wxr_reload_plugins(scene);
  wxr_deserialize_scene(scene, serialization);
  free(serialization);

  return 0;
}

void wxr_set_scene_reload(WXR_Scene *scene) {
  wxr_assert(scene, "Scene is NULL during wxr_set_scene_reload");
  scene->should_reload = 1;
}

WXR_Component_Schema *wxr_create_component_schema() {
  WXR_Component_Schema *schema =
      (WXR_Component_Schema *)malloc(sizeof(WXR_Component_Schema));
  GArray *fields_array =
      g_array_new(FALSE, FALSE, sizeof(WXR_Component_Field *));
  schema->fields = fields_array;
  return schema;
}

// NOLINTNEXTLINE(bugprone-easily-swappable-parameters)
WXR_Component_Field *
wxr_create_component_field(const char *field_name, WXR_Primitive_Type type,
                           WXR_Component_Getter getter,
                           WXR_Component_Setter setter,
                           WXR_Component_Serializer serializer,
                           WXR_Component_Deserializer deserializer) {
  WXR_Component_Field *field =
      (WXR_Component_Field *)malloc(sizeof(WXR_Component_Field));

  field->field_name = wxr_copy_char_ptr(field_name);
  field->type = type;
  field->getter = getter;
  field->setter = setter;
  field->serializer = serializer;
  field->deserializer = deserializer;

  return field;
}

void wxr_destroy_component_schema(WXR_Component_Schema *schema) {
  if (!schema) {
    return;
  }
  wxr_assert(schema->fields,
             "The fields in the schema are NULL during schema destruction");
  for (size_t i = 0; i < schema->fields->len; i++) {
    WXR_Component_Field *field =
        g_array_index(schema->fields, WXR_Component_Field *, i);
    wxr_destroy_component_field(field);
  }
  g_array_free(schema->fields, TRUE);
  free(schema);
}

void wxr_destroy_component_field(WXR_Component_Field *field) {
  free(field->field_name);
  free(field);
}

int wxr_add_field_to_component_schema(WXR_Component_Schema *schema,
                                      const WXR_Component_Field *field) {
  for (size_t i = 0; i < schema->fields->len; i++) {
    WXR_Component_Field *other =
        g_array_index(schema->fields, WXR_Component_Field *, i);
    wxr_assert_abort_value(field != other, 1,
                           "Schema field has been added twice");
    wxr_assert_abort_value(strcmp(field->field_name, other->field_name) != 0, 1,
                           "Schema field has been added twice");
  }
  g_array_append_val(schema->fields, field);
  return 0;
}

WXR_Component_Field *wxr_get_field(const WXR_Component_Schema *schema,
                                   const char *field_name) {
  wxr_assert_abort_value(schema, NULL, "Schema is null during wxr_get_field");
  for (size_t i = 0; i < schema->fields->len; i++) {
    WXR_Component_Field *field =
        g_array_index(schema->fields, WXR_Component_Field *, i);
    if (strcmp(field->field_name, field_name) == 0) {
      return field;
    }
  }
  return NULL;
}

WXR_Component_Getter wxr_get_field_getter(const WXR_Component_Schema *schema,
                                          const char *field_name) {
  wxr_assert_abort_value(schema, NULL, "Schema is null during ts_get_getter");
  WXR_Component_Field *field = wxr_get_field(schema, field_name);
  if (!field) {
    return NULL;
  }
  return field->getter;
}

WXR_Component_Setter wxr_get_field_setter(const WXR_Component_Schema *schema,
                                          const char *field_name) {
  wxr_assert_abort_value(schema, NULL, "Schema is null during ts_get_getter");
  WXR_Component_Field *field = wxr_get_field(schema, field_name);
  if (!field) {
    return NULL;
  }
  return field->setter;
}

WXR_Primitive_Type wxr_get_field_type(const WXR_Component_Schema *schema,
                                      const char *field_name) {
  wxr_assert(schema, "Schema is null during ts_get_getter");
  WXR_Component_Field *field = wxr_get_field(schema, field_name);
  wxr_assert(field, "Field `%s` not found during the wxr_get_field_type",
             field_name);
  return field->type;
}

const void *wxr_get(const WXR_Scene *scene, const void *component,
                    const char *field) {
  wxr_assert_abort_value(scene, NULL, "Scene is null during wxr_get");
  WXR_Component_Handler *handler =
      wxr_find_handler_for_component(scene, component);
  wxr_assert_abort_value(
      handler, NULL, "The component pointer couldn't be found in the scene");

  WXR_Component_Getter getter = wxr_get_field_getter(handler->schema, field);
  wxr_assert_abort_value(getter, NULL, "No getter found for the field `%s`",
                         field);
  return getter(component);
}

int wxr_set(const WXR_Scene *scene, void *component, const char *field,
            const void *data) {
  wxr_assert_abort_value(scene, 1, "Scene is null during wxr_get");
  WXR_Component_Handler *handler =
      wxr_find_handler_for_component(scene, component);
  wxr_assert_abort_value(
      handler, 1, "The component pointer couldn't be found in the scene");

  WXR_Component_Setter setter = wxr_get_field_setter(handler->schema, field);
  wxr_assert_abort_value(setter, 1, "No setter found for the field `%s`",
                         field);
  setter(component, data);
  return 0;
}

WXR_Component_Schema *wxr_get_schema_of_component(const WXR_Scene *scene,
                                                  const void *component) {
  WXR_Component_Handler *handler =
      wxr_find_handler_for_component(scene, component);
  wxr_assert_abort_value(handler, NULL,
                         "Handler is null during wxr_get_schema_of_component");
  wxr_assert_abort_value(handler->schema, NULL,
                         "Schema is null during wxr_get_schema_of_component");
  return handler->schema;
}

void wxr_set_scene_terminate(WXR_Scene *scene) { scene->should_terminate = 1; }

static size_t wxr_get_byte_length(const char *data) {
  wxr_assert_abort_value(data, 0, "Data is NULL during wxr_get_byte_length");
  size_t length;
  memcpy(&length, data, sizeof(size_t));
  return length;
}

char *wxr_serialize_plugin(const WXR_Scene *scene, const char *plugin_id) {
  wxr_assert_abort_value(scene, NULL,
                         "Scene is NULL during wxr_serialize_plugin");
  wxr_assert_abort_value(plugin_id, NULL,
                         "Plugin ID is NULL during wxr_serialize_plugin");

  long plugin_index = wxr_get_plugin_index(scene, plugin_id);
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

char *wxr_serialize_system(const WXR_Scene *scene, const char *system_id) {
  wxr_assert_abort_value(scene, NULL,
                         "Scene is NULL during wxr_serialize_plugin");
  wxr_assert_abort_value(system_id, NULL,
                         "System ID is NULL during wxr_serialize_plugin");

  long system_index = wxr_get_system_index(scene, system_id);
  if (system_index == -1L) {
    return NULL;
  }

  WXR_System_Handler *system_handler =
      g_array_index(scene->systems, WXR_System_Handler *, system_index);

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
wxr_serialize_component_fields(size_t *total_size, const WXR_Scene *scene,
                               const WXR_Component_Handler *handler) {
  wxr_assert_abort_value(scene, NULL,
                         "Scene is NULL during wxr_serialize_component_fields");
  wxr_assert_abort_value(
      handler, NULL,
      "Component Handler is NULL during wxr_serialize_component_fields");

  const GArray *fields_array = handler->schema->fields; // Alias
  GArray *serializations = g_array_new(FALSE, FALSE, sizeof(char *));

  size_t allocation_size = 0;

  for (size_t i = 0; i < fields_array->len; i++) {
    WXR_Component_Field *field =
        g_array_index(fields_array, WXR_Component_Field *, i);
    if (field->serializer) {
      char *serialization = field->serializer(handler->component);
      g_array_append_val(serializations, serialization);
      size_t offset = wxr_get_byte_length(serialization);
      allocation_size += offset;
    }
  }

  *total_size = allocation_size;
  if (allocation_size == 0) {
    g_array_free(serializations, TRUE);
    return NULL;
  }

  char *data = malloc(allocation_size);
  char *iter = data;

  for (size_t i = 0; i < serializations->len; i++) {
    char *serialization = g_array_index(serializations, char *, i);
    size_t offset = wxr_get_byte_length(serialization);
    memcpy(iter, serialization, offset);
    iter += offset;
    free(serialization);
  }

  g_array_free(serializations, TRUE);

  return data;
}

char *wxr_serialize_component(const WXR_Scene *scene, const void *component) {
  wxr_assert_abort_value(scene, NULL,
                         "Scene is NULL during wxr_serialize_component");
  wxr_assert_abort_value(component, NULL,
                         "Component is NULL during wxr_serialize_component");

  WXR_Component_Handler *component_handler =
      wxr_find_handler_for_component(scene, component);
  if (!component_handler) {
    wxr_error("Failed to find the component in the register during "
              "wxr_serialize_component");
    return NULL;
  }

  size_t field_serialization_size = 0;
  char *field_serialization = wxr_serialize_component_fields(
      &field_serialization_size, scene, component_handler);

  size_t allocation_size = sizeof(size_t) + strlen(component_handler->id) + 1 +
                           field_serialization_size;
  char *data = (char *)malloc(allocation_size);
  char *iter = data;

  memcpy(iter, &allocation_size, sizeof(size_t));
  iter += sizeof(size_t);

  memcpy(iter, component_handler->id, strlen(component_handler->id) + 1);
  iter += strlen(component_handler->id) + 1;

  if (field_serialization_size != 0 && field_serialization != NULL) {
    memcpy(iter, field_serialization, field_serialization_size);
  }

  free(field_serialization);

  return data;
}

char *wxr_serialize_entity(const WXR_Scene *scene, const WXR_Entity entity_id) {
  wxr_assert_abort_value(scene, NULL,
                         "Scene is NULL during wxr_serialize_entity");
  wxr_assert_abort_value(entity_id < scene->entity_counter, NULL,
                         "Entity ID is invalid");

  // Compute the length of the component array
  size_t component_counter = 0;
  for (size_t i = 0; i < scene->components->len; i++) {
    WXR_Component_Handler *component_handler =
        g_array_index(scene->components, WXR_Component_Handler *, i);
    if (component_handler->entity == entity_id) {
      component_counter++;
    }
  }

  char **component_serializations = NULL;
  if (component_counter != 0) {
    // Create the component array
    component_serializations =
        (char **)malloc(sizeof(char *) * component_counter);
    // Initialize all the values
    for (size_t i = 0; i < component_counter; i++) {
      component_serializations[i] = NULL;
    }
  }

  size_t allocation = sizeof(size_t);

  // Compute the component array and also compute the allocation length
  size_t component_index = 0;
  for (size_t i = 0; i < scene->components->len; i++) {
    WXR_Component_Handler *component_handler =
        g_array_index(scene->components, WXR_Component_Handler *, i);
    if (component_handler->entity == entity_id) {
      char *component_serialization =
          wxr_serialize_component(scene, component_handler->component);
      // Check for clang-tidy to be sure about the array bounds
      if (component_index < component_counter) {
        component_serializations[component_index++] = component_serialization;
        size_t offset = wxr_get_byte_length(component_serialization);
        allocation += offset;
      } else {
        size_t offset = wxr_get_byte_length(component_serialization);
        allocation += offset;
        free(component_serialization);
      }
    }
  }

  char *data = (char *)malloc(allocation);
  char *iter = data;

  memcpy(iter, &allocation, sizeof(size_t));
  iter += sizeof(size_t);

  for (size_t i = 0; i < component_counter; i++) {
    char *component = component_serializations[i];
    if (!component) {
      continue;
    }
    size_t offset = wxr_get_byte_length(component);
    memcpy(iter, component, offset);
    iter += offset;

    // Cleanup
    free(component);
  }
  free(component_serializations);

  return data;
}

char *wxr_serialize_scene(const WXR_Scene *scene) {
  wxr_assert_abort_value(scene, NULL,
                         "Scene is NULL during wxr_serialize_scene");

  size_t allocation = sizeof(size_t) + sizeof(size_t) + sizeof(size_t);

  // Convert the lengths into proper size_t
  size_t system_size = (size_t)scene->systems->len;
  size_t entity_size = (size_t)scene->entities->len;

  char **system_serializations = (char **)malloc(sizeof(char *) * system_size);
  char **entity_serializations = (char **)malloc(sizeof(char *) * entity_size);

  // Gather all the serializations and figure out how much to allocate in the
  // end
  for (size_t i = 0; i < system_size; i++) {
    WXR_System_Handler *system_handler =
        g_array_index(scene->systems, WXR_System_Handler *, i);
    system_serializations[i] = wxr_serialize_system(scene, system_handler->id);
    size_t offset = wxr_get_byte_length(system_serializations[i]);
    allocation += offset;
  }

  for (size_t i = 0; i < entity_size; i++) {
    WXR_Entity entity_id = g_array_index(scene->entities, WXR_Entity, i);
    entity_serializations[i] = wxr_serialize_entity(scene, entity_id);
    size_t offset = wxr_get_byte_length(entity_serializations[i]);
    allocation += offset;
  }

  // Construct the final serialization
  char *data = (char *)malloc(allocation);
  char *iter = data;

  memcpy(iter, &allocation, sizeof(size_t));
  iter += sizeof(size_t);

  memcpy(iter, &system_size, sizeof(size_t));
  iter += sizeof(size_t);

  memcpy(iter, &entity_size, sizeof(size_t));
  iter += sizeof(size_t);

  for (size_t i = 0; i < system_size; i++) {
    size_t offset = wxr_get_byte_length(system_serializations[i]);
    memcpy(iter, system_serializations[i], offset);
    iter += offset;
    free(system_serializations[i]);
  }

  for (size_t i = 0; i < entity_size; i++) {
    size_t offset = wxr_get_byte_length(entity_serializations[i]);
    memcpy(iter, entity_serializations[i], offset);
    iter += offset;
    free(entity_serializations[i]);
  }

  // Cleanup
  free(system_serializations);
  free(entity_serializations);

  return data;
}

int wxr_deserialize_plugin(WXR_Scene *scene, const char *data) {
  wxr_assert_abort_value(scene, 1,
                         "Scene is NULL during wxr_deserialize_plugin");
  wxr_assert_abort_value(data, 1, "Data is NULL during wxr_deserialize_plugin");

  const char *iter = data + sizeof(size_t);

  wxr_load_plugin(scene, iter);

  return 0;
}

int wxr_deserialize_system(WXR_Scene *scene, const char *data) {
  wxr_assert_abort_value(scene, 1,
                         "Scene is NULL during wxr_deserialize_system");
  wxr_assert_abort_value(data, 1, "Data is NULL during wxr_deserialize_system");

  size_t size = wxr_get_byte_length(data);
  const char *iter = data + sizeof(size_t);
  size -= sizeof(size_t);

  const char *system_name = iter;

  int system_priority = 0;
  memcpy(&system_priority, iter + (sizeof(char) * size - sizeof(int)),
         sizeof(int));

  return wxr_add_system(scene, system_name, system_priority);
}

static int wxr_deserialize_component_fields(WXR_Scene *scene, const char *data,
                                            WXR_Component_Handler *handler,
                                            const char *end) {
  wxr_assert_abort_value(
      scene, 1, "Scene is NULL during wxr_deserialize_component_fields");
  wxr_assert_abort_value(
      data, 1, "Data is NULL during wxr_deserialize_component_fields");
  wxr_assert_abort_value(
      handler, 1,
      "Component Handler is NULL during wxr_deserialize_component_fields");
  wxr_assert_abort_value(
      end, 1, "End marker is NULL during wxr_deserialize_component_fields");

  const char *iter = data;

  const GArray *fields = handler->schema->fields;

  while (iter < end) {
    const size_t size = wxr_get_byte_length(iter);
    const char *field_name = iter + sizeof(size_t);
    const size_t field_name_size = wxr_len_till_null(field_name, sizeof(char));
    const void *field_data = iter + sizeof(size_t) + field_name_size + 1;

    for (size_t i = 0; i < fields->len; i++) {
      WXR_Component_Field *field =
          g_array_index(fields, WXR_Component_Field *, i);
      if (strcmp(field->field_name, field_name) == 0) {
        if (field->deserializer) {
          field->deserializer(handler->component, field_data);
          break;
        }
      }
    }
    iter += size;
  }

  wxr_assert_abort_value(
      iter == end, 1,
      "Deserialization of the component didn't work properly. Data corrupted");

  return 0;
}

int wxr_deserialize_component(WXR_Scene *scene, const WXR_Entity entity,
                              const char *data) {
  wxr_assert_abort_value(scene, 1,
                         "Scene is NULL during wxr_deserialize_component");
  wxr_assert_abort_value(data, 1,
                         "Data is NULL during wxr_deserialize_component");
  const size_t size = wxr_get_byte_length(data);

  const char *component_name = data + sizeof(size_t);

  const char *end = data + size;
  const char *iter = data + sizeof(size_t) + strlen(component_name) + 1;

  void *component_ptr = wxr_add_component(scene, entity, component_name);
  WXR_Component_Handler *handler =
      wxr_find_handler_for_component(scene, component_ptr);
  wxr_assert_abort_value(
      handler, 1,
      "Component couldn't be added during wxr_deserialize_component");

  return wxr_deserialize_component_fields(scene, iter, handler, end);
}

int wxr_deserialize_entity(WXR_Scene *scene, const char *data) {
  wxr_assert_abort_value(scene, 1,
                         "Scene is NULL during wxr_deserialize_entity");
  wxr_assert_abort_value(data, 1, "Data is NULL during wxr_deserialize_entity");

  const size_t size = wxr_get_byte_length(data);

  const WXR_Entity entity = wxr_add_entity(scene);

  const char *end = data + size;
  const char *iter = data + sizeof(size_t);

  while (iter < end) {
    const size_t length = wxr_get_byte_length(iter);
    wxr_deserialize_component(scene, entity, iter);
    iter += length;
  }

  if (iter != end) {
    wxr_error(
        "Invalid component deserialization while deserializing entity %ld",
        entity);
    return 1;
  }

  return 0;
}

int wxr_deserialize_scene(WXR_Scene *scene, const char *data) {
  wxr_assert_abort_value(scene, 1,
                         "Scene is NULL during wxr_deserialize_scene");
  wxr_assert_abort_value(data, 1, "Data is NULL during wxr_deserialize_scene");

  const size_t size = wxr_get_byte_length(data);

  const size_t system_size = wxr_get_byte_length(data + sizeof(size_t));
  const size_t entity_size = wxr_get_byte_length(data + (2 * sizeof(size_t)));

  const char *iter = data + (3 * sizeof(size_t));

  for (size_t i = 0; i < system_size; i++) {
    const size_t length = wxr_get_byte_length(iter);
    int status = wxr_deserialize_system(scene, iter);
    if (status) {
      return status;
    }
    iter += length;
  }

  for (size_t i = 0; i < entity_size; i++) {
    const size_t length = wxr_get_byte_length(iter);
    int status = wxr_deserialize_entity(scene, iter);
    if (status) {
      return status;
    }
    iter += length;
  }

  wxr_assert_abort_value(
      data + size == iter, 1,
      "Length of serialized data was invalid. Potentially corrupted scene");

  return 0;
}

int wxr_serialize_scene_to_file(const WXR_Scene *scene, const char *path) {
  wxr_assert_abort_value(scene, 1,
                         "Scene is NULL during wxr_serialize_scene_to_file");
  wxr_assert_abort_value(path, 1,
                         "Path is NULL during wxr_serialize_scene_to_file");

  // Serialize the scene
  char *data = wxr_serialize_scene(scene);
  wxr_assert_abort_value(
      data, 1, "Failed to serialize scene during wxr_serialize_scene_to_file");

  // Extract the size from the data
  size_t size;
  memcpy(&size, data, sizeof(size_t));

  // Open the file for binary writing
  FILE *file = fopen(path, "wb");
  if (!file) {
    wxr_error("Failed to open file '%s' for writing", path);
    free(data);
    return 1;
  }

  // Write the serialized data to the file
  size_t written = fwrite(data, 1, size, file);
  if (written != size) {
    wxr_error("Failed to write complete data to file '%s' (%zu/%zu bytes "
              "written)",
              path, written, size);
    int status = fclose(file);
    if (status) {
      wxr_error("Also failed to close the file");
      free(data);
      return 1;
    }
    free(data);
    return 1;
  }

  // Close the file
  int status = fclose(file);
  if (status) {
    wxr_warn(
        "Failed to close the file (still the write succeeded). Proceeding");
  }

  // Cleanup
  free(data);

  wxr_debug("Scene serialized to file '%s' (%zu bytes)", path, size);

  return 0;
}

void wxr_deserialize_scene_from_file(WXR_Scene *scene, const char *path) {
  wxr_assert_abort(scene,
                   "Scene is NULL during wxr_deserialize_scene_from_file");
  wxr_assert_abort(path, "Path is NULL during wxr_deserialize_scene_from_file");

  scene->should_load = wxr_copy_char_ptr(path);
}

// Debug Functions

void wxr_print_entities(const WXR_Scene *scene) {
  printf("Active Entities %d:\n", scene->entities->len);
  for (size_t i = 0; i < scene->entities->len; i++) {
    const WXR_Entity entity = g_array_index(scene->entities, WXR_Entity, i);
    printf("- Entity %ld\n", entity);
  }
}

void wxr_print_plugins(const WXR_Scene *scene) {
  printf("Loaded Plugins %d:\n", scene->plugins->len);
  for (size_t i = 0; i < scene->plugins->len; i++) {
    const WXR_Plugin_Handler *plugin =
        g_array_index(scene->plugins, WXR_Plugin_Handler *, i);
    printf("- %s\n", plugin->path);
  }
}

void wxr_print_components(const WXR_Scene *scene) {
  printf("Active Components %d:\n", scene->components->len);
  for (size_t i = 0; i < scene->entities->len; i++) {
    const WXR_Entity entity = g_array_index(scene->entities, WXR_Entity, i);
    printf("- Entity %ld:\n", entity);
    for (size_t j = 0; j < scene->components->len; j++) {
      const WXR_Component_Handler *component =
          g_array_index(scene->components, WXR_Component_Handler *, j);
      if (component->entity == entity) {
        printf("  - %s (%s)\n", component->id, component->plugin->path);
      }
    }
  }
}

void wxr_print_systems(const WXR_Scene *scene) {
  printf("Active Systems %d:\n", scene->systems->len);
  for (size_t i = 0; i < scene->systems->len; i++) {
    const WXR_System_Handler *system =
        g_array_index(scene->systems, WXR_System_Handler *, i);
    printf("- %s (Priority: %d) (%s)\n", system->id, system->priority,
           system->plugin->path);
  }
}
