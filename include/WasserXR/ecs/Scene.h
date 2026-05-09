#ifndef WXR_Scene_H
#define WXR_Scene_H

#include <stddef.h>

// Struct declarations
typedef struct WXR_Scene WXR_Scene;
typedef struct WXR_Component_Schema WXR_Component_Schema;
typedef struct WXR_Component_Field WXR_Component_Field;

// Enum declarations
typedef enum WXR_Primitive_Type {
  WXR_L,
  WXR_F,
  WXR_C,
  WXR_BLOB,
  WXR_S,
  WXR_BLOB_ARRAY
} WXR_Primitive_Type;

// Primitive Declarations
typedef size_t WXR_Entity;
typedef int WXR_Field_Permission;

// Function Prefix Macros
#define CREATOR_FUNCTION_PREFIX "wxr_create_"
#define DESTROYER_FUNCTION_PREFIX "wxr_destroy_"
#define SCHEMA_FUNCTION_PREFIX "wxr_schema_"

#define SYSTEM_FUNCTION_PREFIX "wxr_system_"
#define SYSTEM_SELECTOR_PREFIX "wxr_select_"
#define SYSTEM_ATTACH_PREFIX "wxr_attach_"
#define SYSTEM_DETACH_PREFIX "wxr_detach_"
#define SYSTEM_GROUPS_PREFIX "wxr_groups_"

// Component Functions
typedef void *(*WXR_Component_Creator)();
typedef void (*WXR_Component_Destroyer)(void *);
typedef void (*WXR_Component_Schema_Function)(WXR_Component_Schema *);
typedef const void *(*WXR_Component_Getter)(const void *);
typedef void (*WXR_Component_Setter)(void *, const void *);
typedef char *(*WXR_Component_Serializer)(const void *);
typedef int (*WXR_Component_Deserializer)(void *, const char *);

// System Functions
typedef int WXR_System_Groups;
typedef void (*WXR_System_Function)(WXR_Scene *, WXR_Entity **, const size_t *);
typedef WXR_System_Groups (*WXR_System_Selector)(const WXR_Scene *,
                                               const WXR_Entity);
typedef void (*WXR_System_Attacher)(WXR_Scene *);
typedef void (*WXR_System_Detacher)(WXR_Scene *);

// Permission Bits
#define WXR_Permission_Mask_Serialize 1

// Permission Groups
#define WXR_Permission_All WXR_Permission_Mask_Serialize
#define WXR_Permission_No_Serialize 0

/**
 * Create a new scene object.
 * It allocates all the required memory to store all entities, components and
 * systems.
 * @return Pointer to the newly created scene object
 */
WXR_Scene *wxr_create_scene();

/**
 * Destroy a scene and free all associated memory.
 * @param scene The scene to destroy
 */
void wxr_destroy_scene(WXR_Scene *scene);

/**
 * Resets all the entities and their components. It also removes all systems.
 * All plugins currently loaded persist.
 * @param scene The scene to reset
 */
void wxr_reset_scene(WXR_Scene *scene);

/**
 * Creates an entity in the scene and returns the id of it.
 * @param scene The scene to create it in
 * @return The id of the entity
 */
WXR_Entity wxr_add_entity(WXR_Scene *scene);

/**
 * Removes a previously created entity. With it, it removes all the associated
 * component data.
 * @param scene The scene where the entity that should be removed lives in
 * @param entity_id The entity id that you want to delete
 * @return 0 if it successfully removed the entity and 1 if it didn't remove the
 * entity (may happen if the entity doesn't exist)
 */
int wxr_remove_entity(WXR_Scene *scene, WXR_Entity entity_id);

/**
 * Returns an array of entities currently in the Scene
 * @param size The pointer which will hold the length of the array
 * @param scene The scene that carries the entities
 * @return The array of entities (owned by the caller)
 */
WXR_Entity *wxr_get_entities(size_t *size, const WXR_Scene *scene);

/**
 * Load a plugin into the scene. A more detailed error message will be logged
 * using the logging system in case the plugin fails to be dynamically linked
 * into the program.
 * @param scene The scene to load the plugin into
 * @param plugin_path Path to the plugin shared library
 * @return 0 on success, non-zero on failure
 */
int wxr_load_plugin(WXR_Scene *scene, const char *plugin_name);

/**
 * Unload a plugin from the scene. It destroys all the components and systems
 * associated to them so that when the plugin is loaded back in, you need to
 * recreate all the components and systems. To avoid that see @ref
 * wxr_reload
 * @param scene The scene to unload the plugin from
 * @param plugin_name Name of the plugin to unload
 * @return 0 on success, non-zero on failure
 */
int wxr_unload_plugin(WXR_Scene *scene, const char *plugin_name);

/**
 * Returns an array of plugin names currently in the Scene
 * @param size The pointer which will hold the length of the array
 * @param scene The scene that carries the plugins
 * @return The array of the plugin names, or NULL if no plugins are loaded.
 *         Caller must free each string in the array and the array itself.
 */
char **wxr_get_plugins(size_t *size, const WXR_Scene *scene);

/**
 * Reload all plugins in the scene. It automatically replaces all the systems
 * with the new functions and recreates all the components.
 * This function should not be called within a system or component. This would
 * also replace the system it came from and hence the return address would
 * become invalid. To call this function refer to @ref wxr_set_scene_reload.
 * Note that all statically linked plugins cannot be reloaded at runtime. Hence
 * they won't change their code. But they will still be reconstructed.
 * @param scene The scene whose plugins should be reloaded
 * instead
 * @return 0 on success, non-zero on failure
 */
int wxr_reload(WXR_Scene *scene);

/**
 * Add a component to an entity. This will run the creator function (see @ref
 * WXR_Component_Creator) to create a suitable container. The object that is
 * created by that function will also be returned by this function.
 * @param scene The scene containing the entity
 * @param entity_id The entity to add the component to
 * @param component_id Name of the component type to add
 * @return Pointer to the created component data, or NULL on failure
 */
void *wxr_add_component(WXR_Scene *scene, WXR_Entity entity_id,
                       const char *component_id);

/**
 * Remove a component from an entity. This will completely free the object by
 * running the destroyer function of the component (see @ref
 * WXR_Component_Destroyer).
 * @param scene The scene containing the entity
 * @param entity_id The entity to remove the component from
 * @param component_name Name of the component type to remove
 * @return 0 on success, non-zero on failure
 */
int wxr_remove_component(WXR_Scene *scene, WXR_Entity entity_id,
                        const char *component_id);

/**
 * Returns an array of component names currently attached to the entity
 * @param size The pointer which will hold the length of the array
 * @param scene The scene that carries the plugins
 * @param entity_id The entity from which the components should be queried
 * @return The array of the component names, or NULL if the entity has no
 * components. Caller must free each string in the array and the array itself.
 */
char **wxr_get_components_of_entity(size_t *size, const WXR_Scene *scene,
                                   WXR_Entity entity_id);

/**
 * Get a component from an entity.
 * @param scene The scene containing the entity
 * @param entity_id The entity to get the component from
 * @param component_name Name of the component type to retrieve
 * @return Pointer to the component data, or NULL if not found
 */
void *wxr_entity_get_component(const WXR_Scene *scene, WXR_Entity entity_id,
                              const char *component_id);

/**
 * Add a system to the scene.
 * @param scene The scene to add the system to
 * @param system_name Name of the system to add
 * @param priority Priority value for system execution order (lower values
 * execute first)
 * @return 0 on success, non-zero on failure
 */
int wxr_add_system(WXR_Scene *scene, const char *system_id, int priority);

/**
 * Remove a system from the scene.
 * @param scene The scene to remove the system from
 * @param system_name Name of the system to remove
 * @return 0 on success, non-zero on failure
 */
int wxr_remove_system(WXR_Scene *scene, const char *system_id);

/**
 * Returns an array of system names currently in the Scene
 * @param size The pointer which will hold the length of the array
 * @param scene The scene that carries the systems
 * @return The array of the system names, or NULL if no systems are registered.
 *         Caller must free each string in the array and the array itself.
 */
char **wxr_get_systems(size_t *size, const WXR_Scene *scene);

/**
 * Execute one tick/frame of all systems in the scene. This will call the system
 * function of each system (see @ref WXR_System_Function)
 * @param scene The scene to tick
 * @return 1 on a normal tick run and 0 if the Scene wants be terminated
 */
int wxr_tick_scene(WXR_Scene *scene);

/**
 * Tells the scene to reload at the end of a tick all the systems and components
 * @param scene The scene to destroy
 */
void wxr_set_scene_reload(WXR_Scene *scene);

/**
 * Tells the scene that at the end of the tick it should tell the main loop to
 * terminate.
 * It is the responsibility of the main loop to take this termination
 * response seriously. It can also ignore it.
 * @param scene The scene that should be terminated
 */
void wxr_set_scene_terminate(WXR_Scene *scene);

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
WXR_Entity *wxr_find_entities_with_selector_and_groups(
    size_t *size, WXR_Scene *scene, WXR_System_Selector selector, int group);

/**
 * Create a new component field definition.
 * Fields are the basic building blocks of component schemas, defining
 * individual properties that components can have with their type,
 * size, and access methods.
 * To make life easier there are some convenience macros defined in @ref
 * Macros.h . There is defined for example @ref WXR_SCHEMA_FIELD
 * @param field_name Name of the field
 * @param type Primitive type of the field (WXR_L, WXR_F, WXR_C, WXR_BLOB,
 * WXR_S, WXR_BLOB_ARRAY)
 * @param getter Function pointer to get the field value from a
 * component instance
 * @param setter Function pointer to set the field value on a component
 * instance
 * @param serializer Function that serializes the field in the correctly
 * specified format. If it is NULL, this field will not be serialized.
 * @param deserializer Function that deserializes the field from the correctly
 * specified format. If it is NULL, this field will not be deserialized.
 * @return Pointer to the newly created component field, or NULL on
 * failure
 */
WXR_Component_Field *wxr_create_component_field(
    const char *field_name, WXR_Primitive_Type type, WXR_Component_Getter getter,
    WXR_Component_Setter setter, WXR_Component_Serializer serializer,
    WXR_Component_Deserializer deserializer);

/**
 * Destroy a component field and free its memory.
 * @param field The component field to destroy
 */
void wxr_destroy_component_field(WXR_Component_Field *field);

/**
 * Create a new component schema.
 * A component schema defines the structure of a component type by holding
 * a collection of fields. Schemas are used by the ECS to understand component
 * layout for serialization, reflection, and field access.
 * @return Pointer to the newly created component schema, or NULL on failure
 */
WXR_Component_Schema *wxr_create_component_schema();

/**
 * Destroy a component schema and free its memory.
 * This will also destroy all fields that were added to the schema.
 * @param schema The component schema to destroy
 */
void wxr_destroy_component_schema(WXR_Component_Schema *schema);

/**
 * Add a field to a component schema.
 * This function registers a field definition with the schema, allowing the ECS
 * to track and access this field on component instances.
 * This function is often couples with the @ref wxr_create_component_field
 * function which is why there exists in @ref Macros.h some helper macros like
 * @ref WXR_SCHEMA_FIELD to make life easier.
 * @param schema The schema to add the field to
 * @param field The field to add (ownership is transferred to the schema)
 * @return 0 on success, non-zero on failure
 */
int wxr_add_field_to_component_schema(WXR_Component_Schema *schema,
                                     const WXR_Component_Field *field);

// TODO: Should be made constant for public usage
/**
 * Get the schema for a given component instance.
 * This function looks up the schema that defines the structure of the provided
 * component, allowing you to introspect the component's fields.
 * @param scene The scene containing the component
 * @param component Pointer to the component instance
 * @return Pointer to the component's schema, or NULL if not found
 */
WXR_Component_Schema *wxr_get_schema_of_component(const WXR_Scene *scene,
                                                const void *component);

// TODO: Should be made constant for public usage
/**
 * Get a field from a component schema by name.
 * @param schema The component schema to search
 * @param field Name of the field to retrieve
 * @return Pointer to the field definition, or NULL if not found
 */
WXR_Component_Field *wxr_get_field(const WXR_Component_Schema *schema,
                                 const char *field);

/**
 * Get the getter function for a specific field in a component schema.
 * The getter function can be used to retrieve the field's value from a
 * component instance.
 * @param schema The component schema to search
 * @param field_name Name of the field
 * @return Function pointer to the getter, or NULL if not found
 */
WXR_Component_Getter wxr_get_field_getter(const WXR_Component_Schema *schema,
                                        const char *field_name);

/**
 * Get the setter function for a specific field in a component schema.
 * The setter function can be used to update the field's value on a component
 * instance.
 * @param schema The component schema to search
 * @param field_name Name of the field
 * @return Function pointer to the setter, or NULL if not found
 */
WXR_Component_Setter wxr_get_field_setter(const WXR_Component_Schema *schema,
                                        const char *field_name);

/**
 * Get the primitive type of a specific field in a component schema.
 * @param schema The component schema to search
 * @param field_name Name of the field
 * @return The primitive type (WXR_L, WXR_F, WXR_C, WXR_BLOB, WXR_S, WXR_BLOB_ARRAY)
 */
WXR_Primitive_Type wxr_get_field_type(const WXR_Component_Schema *schema,
                                    const char *field_name);

/**
 * Get the value of a field from a component instance.
 * This is a generic getter that uses the component's schema to look up
 * the appropriate field getter function and retrieve the value.
 * The value returned is by convention the direct pointer to the struct field.
 * It is returned as a constant pointer which means unless you know what you do,
 * the pointer should not be modified directly. Instead you should use the @ref
 * wxr_set function.
 * @param scene The scene containing the component
 * @param component Pointer to the component instance
 * @param field Name of the field to get
 * @return Pointer to the field value, or NULL if not found. The data/pointer is
 * not owned by the caller.
 */
const void *wxr_get(const WXR_Scene *scene, const void *component,
                   const char *field);

/**
 * Set the value of a field on a component instance.
 * This is a generic setter that uses the component's schema to look up
 * the appropriate field setter function and update the value.
 * @param scene The scene containing the component
 * @param component Pointer to the component instance
 * @param field Name of the field to set
 * @param data Pointer to the new value to set. The data is copied in the @ref
 * WXR_Component_Setter function and therefore the caller still owns the data.
 * @return 0 on success, non-zero on failure
 */
int wxr_set(const WXR_Scene *scene, void *component, const char *field,
           const void *data);

/**
 * Serializes the loaded plugin from the Scene
 * @param scene The scene that the plugin is loaded in
 * @param plugin_id The name of the plugin that should be serialized
 * @return The byte data of the plugin. It is prefixed with a `size_t` which
tells you how many bytes are in the serialization. The data is owned by the
caller.
 */
char *wxr_serialize_plugin(const WXR_Scene *scene, const char *plugin_id);

/**
 * Serializes an active system in the Scene.
 * @param scene The scene that the system is currently registered in
 * @param system_id The name of the system that should be serialized
 * @return The byte data of the system. It is prefixed with a `size_t` which
tells you how many bytes are in the serialization. The data is owned by the
caller.
 */
char *wxr_serialize_system(const WXR_Scene *scene, const char *system_id);

/**
 * Serializes an active component in the Scene.
 * @param scene The scene that the component is currently registered in
 * @param component The pointer to the component data.
 * @return The byte data of the component. It is prefixed with a `size_t` which
tells you how many bytes are in the serialization. The data is owned by the
caller.
 */
char *wxr_serialize_component(const WXR_Scene *scene, const void *component);

/**
 * Serializes an entity in the Scene.
 * @param scene The scene that the entity is currently registered in
 * @param entity The entity id of the entity that should be serialized.
 * @return The byte data of the entity. It contains the information about the
entity and it's components. It is prefixed with a `size_t` which tells you how
many bytes are in the serialization. The data is owned by the caller.
 */
char *wxr_serialize_entity(const WXR_Scene *scene, WXR_Entity entity);

/**
 * Serializes the entire scene. The serialization data contains information
 * about the entities, components and systems. Note: It doesn't include
 * information about the plugins. The plugins are not serialized
 * @param scene The scene that should be serialized
 * @return The byte data of the scene. It contains the information about the
 * entities, components, and systems. It is prefixed with a `size_t` which tells
 * you how many bytes are in the serialization. The data is owned by the caller.
 */
char *wxr_serialize_scene(const WXR_Scene *scene);

/**
 * Deserializes a plugin bytestream
 * @param scene The scene in which it will be constructed in
 * @param data The bytestream that should be deserialized
 * @return Status of the deserialization. It returns 1 if the data couldn't be
 * deserialized. If it returns 0 it was successfully deserialized.
 */
int wxr_deserialize_plugin(WXR_Scene *scene, const char *data);

/**
 * Deserializes a system bytestream
 * @param scene The scene in which it will be constructed in
 * @param data The bytestream that should be deserialized
 * @return Status of the deserialization. It returns 1 if the data couldn't be
 * deserialized. If it returns 0 it was successfully deserialized.
 */
int wxr_deserialize_system(WXR_Scene *scene, const char *data);

/**
 * Deserializes an component bytestream
 * @param scene The scene in which it will be constructed in
 * @param entity The entity in which the component should be deserialized in
 * @param data The bytestream that should be deserialized
 * @return Status of the deserialization. It returns 1 if the data couldn't be
 * deserialized. If it returns 0 it was successfully deserialized.
 */
int wxr_deserialize_component(WXR_Scene *scene, WXR_Entity entity,
                             const char *data);

/**
 * Deserializes an entity bytestream
 * @param scene The scene in which it will be constructed in
 * @param data The bytestream that should be deserialized
 * @return Status of the deserialization. It returns 1 if the data couldn't be
 * deserialized. If it returns 0 it was successfully deserialized.
 */
int wxr_deserialize_entity(WXR_Scene *scene, const char *data);

/**
 * Deserializes an entire scene bytestream. This deserializes entities,
 * components, and systems, but does not deserialize plugins.
 * @param scene The scene that should be reconstructed
 * @param data The bytestream that should be deserialized
 * @return Status of the deserialization. It returns 1 if the data couldn't be
 * deserialized. If it returns 0 it was successfully deserialized.
 */
int wxr_deserialize_scene(WXR_Scene *scene, const char *data);

/**
 * Serializes the scene into a file specified. By convention the file type
 * should be `.ts` but could by anything. This serializes entities, components,
 * and systems, but does not serialize plugins.
 * If IO Errors occur, they should be logged into the logging system.
 * @param scene The scene that should be serialized
 * @param path The path to the scene file. The user is responsible to make sure
 * that the path exists. If the file doesn't exist, it will be created. If it
 * already exists, it will be overwritten.
 * @return It returns the status of the file operation
 */
int wxr_serialize_scene_to_file(const WXR_Scene *scene, const char *path);

/**
 * Reads the provided file and deserializes the data from the file into a scene.
 * This deserializes entities, components, and systems, but does not deserialize
 * plugins.
 * @param scene The scene that should be constructed
 * @param path The path to the scene file. If the file or path doesn't exist the
 * function fails and nothing happens
 * @return It returns the status of the file operation
 */
void wxr_deserialize_scene_from_file(WXR_Scene *scene, const char *path);

/** @name Debug Functions
 * Functions for debugging and inspecting scene state
 * @{
 */

/**
 * Print all entities in the scene to stdout.
 * @param scene The scene to print entities from
 */
void wxr_print_entities(const WXR_Scene *scene);

/**
 * Print all loaded plugins in the scene to stdout.
 * @param scene The scene to print plugins from
 */
void wxr_print_plugins(const WXR_Scene *scene);

/**
 * Print all registered components in the scene to stdout.
 * @param scene The scene to print components from
 */
void wxr_print_components(const WXR_Scene *scene);

/**
 * Print all systems in the scene to stdout.
 * @param scene The scene to print systems from
 */
void wxr_print_systems(const WXR_Scene *scene);

/** @} */

#endif
