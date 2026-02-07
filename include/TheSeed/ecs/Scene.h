#include <stddef.h>

#ifndef TS_Scene_H
#define TS_Scene_H

typedef struct TS_Scene_t TS_Scene_t;

#define CREATOR_FUNCION_PREFIX "ts_create_"
#define DESTROYER_FUNCION_PREFIX "ts_destroy_"
#define SYSTEM_FUNCTION_PREFIX "ts_system_"
#define SYSTEM_SELECTOR_PREFIX "ts_select_"

typedef void *(*TS_Component_Creator)(void);
typedef void (*TS_Component_Destroyer)(void *);
typedef void (*TS_System_Function)(TS_Scene_t *, size_t *, size_t);
typedef int (*TS_System_Selector)(TS_Scene_t *, const size_t);

/**
 * Create a new scene object.
 * It allocates all the required memory to store all entities, components and
 * systems.
 * @return Pointer to the newly created scene object
 */
TS_Scene_t *ts_create_scene();

/**
 * Creates an entity in the scene and returns the id of it.
 * @param scene The scene to create it in
 * @return The id of the entity
 */
size_t ts_add_entity(TS_Scene_t *scene);

/**
 * Removes a previously created entity. With it, it removes all the associated
 * component data.
 * @param scene The scene where the entity that should be removed lives in
 * @param entity_id The entity id that you want to delete
 * @return 0 if it successfully removed the entity and 1 if it didn't remove the
 * entity (may happen if the entity doesn't exist)
 */
int ts_remove_entity(TS_Scene_t *scene, const size_t entity_id);

/**
 * Load a plugin into the scene.
 * @param scene The scene to load the plugin into
 * @param plugin_path Path to the plugin shared library
 * @return 0 on success, non-zero on failure
 */
int ts_load_plugin(TS_Scene_t *scene, const char *plugin_path);

/**
 * Unload a plugin from the scene. It destroys all the components and systems
 * associated to them so that when the plugin is loaded back in, you need to
 * recreate all the components and systems. To avoid that see ts_reload_plugin
 * @param scene The scene to unload the plugin from
 * @param plugin_name Name of the plugin to unload
 * @return 0 on success, non-zero on failure
 */
int ts_unload_plugin(TS_Scene_t *scene, const char *plugin_name);

/**
 * Reload a certain plugin in the scene. It automatically replaces all the
 * systems with the new functions and recreates all the components
 * @param scene The scene whose plugins should be reloaded
 * @param plugin_name The name of the plugin that should be replaced
 * @param new_plugin_name The name of the new plugin that should be loaded
 * instead
 * @return 0 on success, non-zero on failure
 */
int ts_reload_plugin(TS_Scene_t *scene, const char *plugin_name,
                     const char *new_plugin_name);

/**
 * Reload all plugins in the scene. It automatically replaces all the systems
 * with the new functions and recreates all the components
 * @param scene The scene whose plugins should be reloaded
 * instead
 * @return 0 on success, non-zero on failure
 */
int ts_reload_all_plugins(TS_Scene_t *scene);

/**
 * Add a component to an entity.
 * @param scene The scene containing the entity
 * @param entity_id The entity to add the component to
 * @param component_name Name of the component type to add
 * @return 0 on success, non-zero on failure
 */
int ts_add_component(TS_Scene_t *scene, const size_t entity_id,
                     const char *component_name);

/**
 * Remove a component from an entity.
 * @param scene The scene containing the entity
 * @param entity_id The entity to remove the component from
 * @param component_name Name of the component type to remove
 * @return 0 on success, non-zero on failure
 */
int ts_remove_component(TS_Scene_t *scene, const size_t entity_id,
                        const char *component_name);

/**
 * Get a component from an entity.
 * @param scene The scene containing the entity
 * @param entity_id The entity to get the component from
 * @param component_name Name of the component type to retrieve
 * @return Pointer to the component data, or NULL if not found
 */
void *ts_entity_get_component(TS_Scene_t *scene, const size_t entity_id,
                              const char *component_name);

/**
 * Add a system to the scene.
 * @param scene The scene to add the system to
 * @param system_name Name of the system to add
 * @param priority Priority value for system execution order (lower values
 * execute first)
 * @return 0 on success, non-zero on failure
 */
int ts_add_system(TS_Scene_t *scene, const char *system_name, int priority);

/**
 * Remove a system from the scene.
 * @param scene The scene to remove the system from
 * @param system_name Name of the system to remove
 * @return 0 on success, non-zero on failure
 */
int ts_remove_system(TS_Scene_t *scene, const char *system_name);

/**
 * Sort systems in the scene by their priority values.
 * @param scene The scene whose systems should be sorted
 */
void ts_sort_systems(TS_Scene_t *scene);

/**
 * Execute one tick/frame of all systems in the scene.
 * @param scene The scene to tick
 */
void ts_tick_scene(TS_Scene_t *scene);

/**
 * Destroy a scene and free all associated memory.
 * @param scene The scene to destroy
 */
void ts_destroy_scene(TS_Scene_t *scene);

/** @name Debug Functions
 * Functions for debugging and inspecting scene state
 * @{
 */

/**
 * Print all entities in the scene to stdout.
 * @param scene The scene to print entities from
 */
void ts_print_entities(TS_Scene_t *scene);

/**
 * Print all loaded plugins in the scene to stdout.
 * @param scene The scene to print plugins from
 */
void ts_print_plugins(TS_Scene_t *scene);

/**
 * Print all registered components in the scene to stdout.
 * @param scene The scene to print components from
 */
void ts_print_components(TS_Scene_t *scene);

/**
 * Print all systems in the scene to stdout.
 * @param scene The scene to print systems from
 */
void ts_print_systems(TS_Scene_t *scene);

/** @} */

#endif
