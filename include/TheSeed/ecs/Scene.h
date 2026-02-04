#include <stddef.h>

#ifndef TS_Scene_H
#define TS_Scene_H

typedef struct TS_Scene_t TS_Scene_t;

TS_Scene_t *ts_create_scene();

size_t ts_add_entity(TS_Scene_t *);
int ts_entity_exists(const TS_Scene_t *, const size_t);
long ts_get_entity_index(const TS_Scene_t *, const size_t);
int ts_remove_entity(TS_Scene_t *, const size_t);

int ts_load_plugin(TS_Scene_t *, const char *);
int ts_plugin_exists(const TS_Scene_t, const char *);
int ts_reload_plugin(TS_Scene_t *);

int ts_add_component(TS_Scene_t *, const size_t, const char *);
int ts_remove_component(TS_Scene_t *, const int, const char *);

int ts_add_system(TS_Scene_t *, const char *);
int ts_remove_system(TS_Scene_t *, const char *);

void ts_destroy_scene(TS_Scene_t *);

// Debug functions

void ts_print_entities(TS_Scene_t *);
void ts_print_plugins(TS_Scene_t *);
void ts_print_components(TS_Scene_t *);
void ts_print_systems(TS_Scene_t *);

#endif
