#ifndef TS_Scene_H
#define TS_Scene_H

#include <stddef.h>

// Struct declarations
typedef struct TS_Scene TS_Scene;
typedef struct TS_Component_Schema TS_Component_Schema;
typedef struct TS_Component_Field TS_Component_Field;

// Enum declarations
typedef enum TS_Primitive_Type {
  TS_L,
  TS_F,
  TS_C,
  TS_BLOB,
  TS_S,
  TS_BLOB_ARRAY
} TS_Primitive_Type;

// Primitive Declarations
typedef size_t TS_Entity;
typedef int TS_Field_Permission;

// Function Prefix Macros
#define CREATOR_FUNCION_PREFIX "ts_create_"
#define DESTROYER_FUNCION_PREFIX "ts_destroy_"
#define SCHEMA_FUNCTION_PREFIX "ts_schema_"

#define SYSTEM_FUNCTION_PREFIX "ts_system_"
#define SYSTEM_SELECTOR_PREFIX "ts_select_"
#define SYSTEM_ATTACH_PREFIX "ts_attach_"
#define SYSTEM_DETACH_PREFIX "ts_detach_"
#define SYSTEM_GROUPS_PREFIX "ts_groups_"

// Component Functions
typedef void *(*TS_Component_Creator)();
typedef void (*TS_Component_Destroyer)(void *);
typedef void (*TS_Component_Schema_Function)(TS_Component_Schema *);
typedef void *(*TS_Component_Getter)(void *);
typedef void (*TS_Component_Setter)(void *, void *);

// System Functions
typedef int TS_System_Groups;
typedef void (*TS_System_Function)(TS_Scene *, TS_Entity **, const size_t *);
typedef TS_System_Groups (*TS_System_Selector)(TS_Scene *, const TS_Entity);
typedef void (*TS_System_Attacher)(TS_Scene *);
typedef void (*TS_System_Detacher)(TS_Scene *);

// Permission Bits
#define TS_Permission_Mask_Serialize 1

// Permission Groups
#define TS_Permission_All TS_Permission_Mask_Serialize
#define TS_Permission_No_Serialize 0

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
TS_Entity ts_add_entity(TS_Scene *scene);

/**
 * Removes a previously created entity. With it, it removes all the associated
 * component data.
 * @param scene The scene where the entity that should be removed lives in
 * @param entity_id The entity id that you want to delete
 * @return 0 if it successfully removed the entity and 1 if it didn't remove the
 * entity (may happen if the entity doesn't exist)
 */
int ts_remove_entity(TS_Scene *scene, TS_Entity entity_id);

/**
 * Returns an array of entities currently in the Scene
 * @param size The pointer which will hold the length of the array
 * @param scene The scene that carries the entities
 * @return The array of entities
 */
TS_Entity *ts_get_entities(size_t *size, const TS_Scene *scene);

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
 * Returns an array of plugin names currently in the Scene
 * @param size The pointer which will hold the length of the array
 * @param scene The scene that carries the plugins
 * @return The array of the plugin names
 */
char **ts_get_plugins(size_t *size, const TS_Scene *scene);

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
int ts_add_component(TS_Scene *scene, TS_Entity entity_id,
                     const char *component_id);

/**
 * Remove a component from an entity.
 * @param scene The scene containing the entity
 * @param entity_id The entity to remove the component from
 * @param component_name Name of the component type to remove
 * @return 0 on success, non-zero on failure
 */
int ts_remove_component(TS_Scene *scene, TS_Entity entity_id,
                        const char *component_id);

/**
 * Returns an array of component names currently attached to the entity
 * @param size The pointer which will hold the length of the array
 * @param scene The scene that carries the plugins
 * @param entity_id The entity from which the components should be queried
 * @return The array of the component names
 */
char **ts_get_components_of_entity(size_t *size, const TS_Scene *scene,
                                   TS_Entity entity_id);

/**
 * Get a component from an entity.
 * @param scene The scene containing the entity
 * @param entity_id The entity to get the component from
 * @param component_name Name of the component type to retrieve
 * @return Pointer to the component data, or NULL if not found
 */
void *ts_entity_get_component(TS_Scene *scene, TS_Entity entity_id,
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
 * Returns an array of system names currently in the Scene
 * @param size The pointer which will hold the length of the array
 * @param scene The scene that carries the systems
 * @return The array of the system names
 */
char **ts_get_systems(size_t *size, const TS_Scene *scene);

/**
 * Execute one tick/frame of all systems in the scene.
 * @param scene The scene to tick
 * @return 1 on a normal tick run and 0 if the Scene should be terminated
 */
int ts_tick_scene(TS_Scene *scene);

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

/**
 * Tells the scene that at the end of the tick it should tell the main loop to
 * terminate.
 * It is the responsibility of the main loop to take this termination
 * response seriously. It can also ignore it.
 * @param scene The scene that should be terminated
 */
void ts_set_scene_terminate(TS_Scene *scene);

/**
 * Find all entities that match a given selector function and group criteria.
 * This function evaluates each entity in the scene using the provided selector
 * function and returns an array of entities that match the specified group.
 * @param size Output parameter that will be set to the number of entities found
 * @param scene The scene to search for entities
 * @param selector The selector function to evaluate entities
 * @param group The group mask to filter entities by
 * @return Dynamically allocated array of entity IDs (caller must free)
 */
TS_Entity *ts_find_entities_with_selector_and_groups(
    size_t *size, TS_Scene *scene, TS_System_Selector selector, int group);

/**
 * Create a new component field definition.
 * Fields are the basic building blocks of component schemas, defining individual
 * properties that components can have with their type, size, and access methods.
 * @param field_name Name of the field
 * @param size Size of the field data in bytes
 * @param type Primitive type of the field (TS_L, TS_F, TS_C, TS_BLOB, TS_S, TS_BLOB_ARRAY)
 * @param permission Permission flags controlling serialization and other behaviors
 * @param getter Function pointer to get the field value from a component instance
 * @param setter Function pointer to set the field value on a component instance
 * @return Pointer to the newly created component field, or NULL on failure
 */
TS_Component_Field *ts_create_component_field(char *field_name, size_t size,
                                              TS_Primitive_Type type,
                                              TS_Field_Permission permission,
                                              TS_Component_Getter getter,
                                              TS_Component_Setter setter);

/**
 * Destroy a component field and free its memory.
 * @param field The component field to destroy
 */
void ts_destroy_component_field(TS_Component_Field *field);

/**
 * Create a new component schema.
 * A component schema defines the structure of a component type by holding
 * a collection of fields. Schemas are used by the ECS to understand component
 * layout for serialization, reflection, and field access.
 * @return Pointer to the newly created component schema, or NULL on failure
 */
TS_Component_Schema *ts_create_component_schema();

/**
 * Destroy a component schema and free its memory.
 * This will also destroy all fields that were added to the schema.
 * @param schema The component schema to destroy
 */
void ts_destroy_component_schema(TS_Component_Schema *schema);

/**
 * Add a field to a component schema.
 * This function registers a field definition with the schema, allowing the ECS
 * to track and access this field on component instances.
 * @param schema The schema to add the field to
 * @param field The field to add (ownership is transferred to the schema)
 * @return 0 on success, non-zero on failure
 */
int ts_add_field_to_component_schema(TS_Component_Schema *schema,
                                     TS_Component_Field *field);

/**
 * Get the schema for a given component instance.
 * This function looks up the schema that defines the structure of the provided
 * component, allowing you to introspect the component's fields.
 * @param scene The scene containing the component
 * @param component Pointer to the component instance
 * @return Pointer to the component's schema, or NULL if not found
 */
TS_Component_Schema *ts_get_schema_of_component(TS_Scene *scene,
                                                void *component);

/**
 * Get a field from a component schema by name.
 * @param schema The component schema to search
 * @param field Name of the field to retrieve
 * @return Pointer to the field definition, or NULL if not found
 */
TS_Component_Field *ts_get_field(TS_Component_Schema *schema, char *field);

/**
 * Get the getter function for a specific field in a component schema.
 * The getter function can be used to retrieve the field's value from a component instance.
 * @param schema The component schema to search
 * @param field_name Name of the field
 * @return Function pointer to the getter, or NULL if not found
 */
TS_Component_Getter ts_get_field_getter(TS_Component_Schema *schema,
                                        char *field_name);

/**
 * Get the setter function for a specific field in a component schema.
 * The setter function can be used to update the field's value on a component instance.
 * @param schema The component schema to search
 * @param field_name Name of the field
 * @return Function pointer to the setter, or NULL if not found
 */
TS_Component_Setter ts_get_field_setter(TS_Component_Schema *schema,
                                        char *field_name);

/**
 * Get the permission flags for a specific field in a component schema.
 * Permission flags control behaviors like serialization access.
 * @param schema The component schema to search
 * @param field_name Name of the field
 * @return Permission flags for the field
 */
TS_Field_Permission ts_get_field_permission(TS_Component_Schema *schema,
                                            char *field_name);

/**
 * Get the size in bytes of a specific field in a component schema.
 * @param schema The component schema to search
 * @param field_name Name of the field
 * @return Size of the field data in bytes
 */
size_t ts_get_field_size(TS_Component_Schema *schema, char *field_name);

/**
 * Get the primitive type of a specific field in a component schema.
 * @param schema The component schema to search
 * @param field_name Name of the field
 * @return The primitive type (TS_L, TS_F, TS_C, TS_BLOB, TS_S, TS_BLOB_ARRAY)
 */
TS_Primitive_Type ts_get_field_type(TS_Component_Schema *schema,
                                    char *field_name);

/**
 * Get the value of a field from a component instance.
 * This is a generic getter that uses the component's schema to look up
 * the appropriate field getter function and retrieve the value.
 * @param scene The scene containing the component
 * @param component Pointer to the component instance
 * @param field Name of the field to get
 * @return Pointer to the field value, or NULL if not found
 */
void *ts_get(TS_Scene *scene, void *component, char *field);

/**
 * Set the value of a field on a component instance.
 * This is a generic setter that uses the component's schema to look up
 * the appropriate field setter function and update the value.
 * @param scene The scene containing the component
 * @param component Pointer to the component instance
 * @param field Name of the field to set
 * @param data Pointer to the new value to set
 * @return 0 on success, non-zero on failure
 */
int ts_set(TS_Scene *scene, void *component, char *field, void *data);

/**
 * Serializes the loaded plugin from the Scene
 * @param scene The scene that the plugin is loaded in
 * @param plugin_id The name of the plugin that should be serialized
 * @return The byte data of the plugin. It is prefixed with a `size_t` which
tells you how many bytes are in the serialization. The data is owned by the
caller.
 */
char *ts_serialize_plugin(const TS_Scene *scene, const char *plugin_id);

/**
 * Serializes an active system in the Scene.
 * @param scene The scene that the system is currently registered in
 * @param system_id The name of the system that should be serialized
 * @return The byte data of the system. It is prefixed with a `size_t` which
tells you how many bytes are in the serialization. The data is owned by the
caller.
 */
char *ts_serialize_system(const TS_Scene *scene, const char *system_id);

/**
 * Serializes an active component in the Scene.
 * @param scene The scene that the component is currently registered in
 * @param component The pointer to the component data.
 * @return The byte data of the component. It is prefixed with a `size_t` which
tells you how many bytes are in the serialization. The data is owned by the
caller.
 */
char *ts_serialize_component(const TS_Scene *scene, const void *component);

/**
 * Serializes an entity in the Scene.
 * @param scene The scene that the entity is currently registered in
 * @param entity The entity id of the entity that should be serialized.
 * @return The byte data of the entity. It contains the information about the
entity and it's components. It is prefixed with a `size_t` which tells you how
many bytes are in the serialization. The data is owned by the caller.
 */
char *ts_serialize_entity(const TS_Scene *scene, TS_Entity entity);

/**
 * Serializes the entire scene.
 * @param scene The scene that should be serialized
 * @return The byte data of the scene. It contains the information about the
entities, components, systems and plugins. It is prefixed with a `size_t` which
tells you how many bytes are in the serialization. The data is owned by the
caller.
 */
char *ts_serialize_scene(const TS_Scene *scene);

/**
 * Serializes the scene into a file specified. By convention the file type
should be `.ts` but could by anything.
 * @param scene The scene that should be serialized
 * @param path The path to the scene file. The user is responsible to make sure
that the path exists. If the file doesn't exist, it will be created. If it
already exists, it will be overwritten.
 * @return It returns the status of the file operation
  */
int ts_serialize_scene_to_file(const TS_Scene *scene, const char *path);

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
