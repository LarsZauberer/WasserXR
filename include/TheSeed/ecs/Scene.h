#include <stddef.h>

#ifndef TS_Scene_H
#define TS_Scene_H

typedef struct TS_Scene TS_Scene;
typedef struct TS_Serialization TS_Serialization;
typedef size_t TS_Entity;

#define CREATOR_FUNCION_PREFIX "ts_create_"
#define DESTROYER_FUNCION_PREFIX "ts_destroy_"
#define SERIALIZER_FUNCTION_PREFIX "ts_serialize_"
#define DESERIALIZER_FUNCTION_PREFIX "ts_deserialize_"
#define ACTIVATOR_FUNCTION_PREFIX "ts_activate_"
#define SYSTEM_FUNCTION_PREFIX "ts_system_"
#define SYSTEM_SELECTOR_PREFIX "ts_select_"
#define SYSTEM_ATTACH_PREFIX "ts_attach_"
#define SYSTEM_DETACH_PREFIX "ts_detach_"
#define SYSTEM_GROUPS_PREFIX "ts_groups_"

typedef void *(*TS_Component_Creator)();
typedef void (*TS_Component_Destroyer)(void *);
typedef void (*TS_Component_Serializer)(void *, TS_Serialization *);
typedef void (*TS_Component_Deserializer)(void *, TS_Serialization *);
typedef void (*TS_Component_Activator)(void *);

typedef int TS_System_Groups;
typedef void (*TS_System_Function)(TS_Scene *, TS_Entity **, const size_t *);
typedef TS_System_Groups (*TS_System_Selector)(TS_Scene *, const TS_Entity);
typedef void (*TS_System_Attacher)();
typedef void (*TS_System_Detacher)();

/**
 * Create a new scene object.
 * It allocates all the required memory to store all entities, components and
 * systems.
 * @return Pointer to the newly created scene object
 */
TS_Scene *ts_create_scene();

/**
 * Creates an entity in the scene and returns the id of it.
 * @param scene The scene to create it in
 * @return The id of the entity
 */
size_t ts_add_entity(TS_Scene *scene);

/**
 * Removes a previously created entity. With it, it removes all the associated
 * component data.
 * @param scene The scene where the entity that should be removed lives in
 * @param entity_id The entity id that you want to delete
 * @return 0 if it successfully removed the entity and 1 if it didn't remove the
 * entity (may happen if the entity doesn't exist)
 */
int ts_remove_entity(TS_Scene *scene, size_t entity_id);

/**
 * Load a plugin into the scene.
 * @param scene The scene to load the plugin into
 * @param plugin_path Path to the plugin shared library
 * @return 0 on success, non-zero on failure
 */
int ts_load_plugin(TS_Scene *scene, const char *plugin_name);

/**
 * Unload a plugin from the scene. It destroys all the components and systems
 * associated to them so that when the plugin is loaded back in, you need to
 * recreate all the components and systems. To avoid that see ts_reload_plugin
 * @param scene The scene to unload the plugin from
 * @param plugin_name Name of the plugin to unload
 * @return 0 on success, non-zero on failure
 */
int ts_unload_plugin(TS_Scene *scene, const char *plugin_name);

/**
 * Reload a certain plugin in the scene. It automatically replaces all the
 * systems with the new functions and recreates all the components
 * @param scene The scene whose plugins should be reloaded
 * @param plugin_name The name of the plugin that should be replaced
 * @param new_plugin_name The name of the new plugin that should be loaded
 * instead
 * @return 0 on success, non-zero on failure
 */
int ts_reload_plugin(TS_Scene *scene, const char *plugin_path,
                     const char *new_plugin_path);

/**
 * Reload all plugins in the scene. It automatically replaces all the systems
 * with the new functions and recreates all the components.
 * This function should not be called within a system or component. This would
 * also replace the system it came from and hence the return address would
 * become invalid.
 * @param scene The scene whose plugins should be reloaded
 * instead
 * @return 0 on success, non-zero on failure
 */
int ts_reload_all_plugins(TS_Scene *scene);

/**
 * Add a component to an entity.
 * @param scene The scene containing the entity
 * @param entity_id The entity to add the component to
 * @param component_name Name of the component type to add
 * @return 0 on success, non-zero on failure
 */
int ts_add_component(TS_Scene *scene, size_t entity_id,
                     const char *component_id, TS_Serialization *serialization);

/**
 * Remove a component from an entity.
 * @param scene The scene containing the entity
 * @param entity_id The entity to remove the component from
 * @param component_name Name of the component type to remove
 * @return 0 on success, non-zero on failure
 */
int ts_remove_component(TS_Scene *scene, size_t entity_id,
                        const char *component_id);

/**
 * Get a component from an entity.
 * @param scene The scene containing the entity
 * @param entity_id The entity to get the component from
 * @param component_name Name of the component type to retrieve
 * @return Pointer to the component data, or NULL if not found
 */
void *ts_entity_get_component(TS_Scene *scene, size_t entity_id,
                              const char *component_id);

/**
 * Add a system to the scene.
 * @param scene The scene to add the system to
 * @param system_name Name of the system to add
 * @param priority Priority value for system execution order (lower values
 * execute first)
 * @return 0 on success, non-zero on failure
 */
int ts_add_system(TS_Scene *scene, const char *system_id, int priority);

/**
 * Remove a system from the scene.
 * @param scene The scene to remove the system from
 * @param system_name Name of the system to remove
 * @return 0 on success, non-zero on failure
 */
int ts_remove_system(TS_Scene *scene, const char *system_id);

/**
 * Sort systems in the scene by their priority values.
 * @param scene The scene whose systems should be sorted
 */
void ts_sort_systems(TS_Scene *scene);

/**
 * Execute one tick/frame of all systems in the scene.
 * @param scene The scene to tick
 */
void ts_tick_scene(TS_Scene *scene);

/**
 * Destroy a scene and free all associated memory.
 * @param scene The scene to destroy
 */
void ts_destroy_scene(TS_Scene *scene);

/**
 * Tells the scene to reload at the end of a tick all the systems and components
 * @param scene The scene to destroy
 */
void ts_set_scene_reload(TS_Scene *scene);

void *ts_get_serialization(TS_Serialization *serialization, char *name);

int ts_set_serialization(TS_Serialization *serialization, char *name,
                         size_t size, void *data);

/** @name Debug Functions
 * Functions for debugging and inspecting scene state
 * @{
 */

/**
 * Print all entities in the scene to stdout.
 * @param scene The scene to print entities from
 */
void ts_print_entities(TS_Scene *scene);

/**
 * Print all loaded plugins in the scene to stdout.
 * @param scene The scene to print plugins from
 */
void ts_print_plugins(TS_Scene *scene);

/**
 * Print all registered components in the scene to stdout.
 * @param scene The scene to print components from
 */
void ts_print_components(TS_Scene *scene);

/**
 * Print all systems in the scene to stdout.
 * @param scene The scene to print systems from
 */
void ts_print_systems(TS_Scene *scene);

/** @} */

#endif
