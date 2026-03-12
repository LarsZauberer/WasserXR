#ifndef _TS_SCENE_INTERNAL_H
#define _TS_SCENE_INTERNAL_H

#include <TheSeed/ecs/Scene.h>
#include <glib.h>

typedef struct TS_Plugin_Handler TS_Plugin_Handler;
typedef struct TS_Component_Handler TS_Component_Handler;
typedef struct TS_System_Handler TS_System_Handler;
typedef struct TS_Component_Serialization TS_Component_Serialization;
typedef struct TS_Component_Serialization_Item TS_Component_Serialization_Item;

struct TS_Plugin_Handler {
  char *path;
  void *fd;
};

struct TS_Component_Handler {
  char *id;
  TS_Entity entity;
  TS_Plugin_Handler *plugin;
  TS_Component_Destroyer destroyer;
  TS_Component_Schema *schema;
  void *component;
};

struct TS_System_Handler {
  char *id;
  int priority;
  int active;
  TS_Plugin_Handler *plugin;
  TS_System_Groups *groups;
  TS_System_Selector selector;
  TS_System_Attacher attacher;
  TS_System_Detacher detacher;
  TS_System_Function system;
};

struct TS_Component_Serialization_Item {
  char *field_name;
  void *data;
};

struct TS_Component_Serialization {
  TS_Entity entity_id;
  char *component_name;
  GArray *fields;
};

struct TS_Scene {
  GArray *plugins;
  TS_Entity entity_counter; // Entities < entity_counter
  GArray *entities;
  GArray *components;
  GArray *systems;
  int should_reload;
  int should_terminate;
};

struct TS_Component_Schema {
  GArray *fields;
};

struct TS_Component_Field {
  char *field_name;
  size_t size;
  TS_Primitive_Type type;
  TS_Field_Permission permission;
  TS_Component_Getter getter;
  TS_Component_Setter setter;
};

// Functions

// Indexers
long ts_get_entity_index(const TS_Scene *scene, TS_Entity entity);

long ts_get_plugin_index(const TS_Scene *scene, const char *path);

long ts_get_system_index(TS_Scene *scene, const char *system_id);

long ts_get_component_index(TS_Scene *scene, TS_Entity entity,
                            const char *component_id);

TS_Component_Handler *ts_find_handler_for_component(TS_Scene *scene,
                                                    void *component);

// RAII of Component Serialization
TS_Component_Serialization *
ts_create_component_serialization(TS_Entity entity_id, char *component_name);

void ts_destroy_component_serialization(
    TS_Component_Serialization *serialization);

TS_Component_Serialization_Item *
ts_create_component_serialization_item(char *field_name, void *data,
                                       size_t size, TS_Primitive_Type type);

void ts_destroy_component_serialization_item(
    TS_Component_Serialization_Item *item);

// Serialization
TS_Component_Serialization *
ts_serialize_component(TS_Component_Handler *handler);

int ts_deserialize_component(TS_Scene *scene, TS_Component_Handler *handler,
                             TS_Component_Serialization *serialization);

// ECS functions

/**
 * Sort systems in the scene by their priority values.
 * @param scene The scene whose systems should be sorted
 */
void ts_sort_systems(TS_Scene *scene);

void *ts_get_abi_symbol_from_plugin(const TS_Scene *scene,
                                    const TS_Plugin_Handler *handler,
                                    const char *prefix, const char *symbol);
void *ts_get_abi_symbol(TS_Plugin_Handler **handler, const TS_Scene *scene,
                        const char *prefix, const char *symbol);

#endif
