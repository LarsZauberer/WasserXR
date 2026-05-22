/*
 * Copyright (c) 2026 Ian Wasser
 * Licensed under the WasserXR License.
 * You may not use this file except in compliance with the License.
 * See LICENSE.md for details.
 */

#ifndef WXR_SCENE_INTERNAL_H
#define WXR_SCENE_INTERNAL_H

#include <WasserXR/ecs/Scene.h>
#include <glib.h>

typedef struct WXR_Plugin_Handler WXR_Plugin_Handler;
typedef struct WXR_Component_Handler WXR_Component_Handler;
typedef struct WXR_System_Handler WXR_System_Handler;
typedef struct WXR_Component_Serialization WXR_Component_Serialization;
typedef struct WXR_Component_Serialization_Item
    WXR_Component_Serialization_Item;

struct WXR_Plugin_Handler {
  char *path;
  void *fd;
};

struct WXR_Component_Handler {
  char *id;
  WXR_Entity entity;
  WXR_Plugin_Handler *plugin;
  WXR_Component_Destroyer destroyer;
  WXR_Component_Schema *schema;
  void *component;
};

struct WXR_System_Handler {
  char *id;
  int priority;
  int active;
  WXR_Plugin_Handler *plugin;
  WXR_System_Groups *groups;
  WXR_System_Selector selector;
  WXR_System_Attacher attacher;
  WXR_System_Detacher detacher;
  WXR_System_Function system;
};

struct WXR_Scene {
  GArray *plugins;
  WXR_Entity entity_counter; // Entities < entity_counter
  GArray *entities;
  GArray *components;
  GArray *systems;
  int should_reload;
  int should_terminate;
  char *should_load;
};

struct WXR_Component_Schema {
  GArray *fields;
};

struct WXR_Component_Field {
  char *field_name;
  WXR_Primitive_Type type;
  WXR_Component_Getter getter;
  WXR_Component_Setter setter;
  WXR_Component_Serializer serializer;
  WXR_Component_Deserializer deserializer;
};

// Functions

// Indexers
long wxr_get_entity_index(const WXR_Scene *scene, WXR_Entity entity);

long wxr_get_plugin_index(const WXR_Scene *scene, const char *path);

long wxr_get_system_index(const WXR_Scene *scene, const char *system_id);

long wxr_get_component_index(const WXR_Scene *scene, WXR_Entity entity,
                             const char *component_id);

WXR_Component_Handler *wxr_find_handler_for_component(const WXR_Scene *scene,
                                                      const void *component);

// ECS functions

/**
 * Sort systems in the scene by their priority values.
 * @param scene The scene whose systems should be sorted
 */
void wxr_sort_systems(WXR_Scene *scene);

void *wxr_get_abi_symbol_from_plugin(const WXR_Scene *scene,
                                     const WXR_Plugin_Handler *handler,
                                     const char *prefix, const char *symbol);
void *wxr_get_abi_symbol(WXR_Plugin_Handler **handler, const WXR_Scene *scene,
                         const char *prefix, const char *symbol);

void wxr_reload_plugins(WXR_Scene *scene);

#endif
