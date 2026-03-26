#ifndef TS_MODEL_H
#define TS_MODEL_H

#include "TheSeed/ecs/Scene.h"

typedef struct TS_Model TS_Model;

void *ts_create_TS_Model();
void ts_destroy_TS_Model(void *ptr);
void ts_schema_TS_Model(TS_Component_Schema *schema);

void *ts_get_TS_Model_model_name(const void *component);
void ts_set_TS_Model_model_name(void *component, const void *data);
void *ts_get_TS_Model_shader_name(const void *component);
void ts_set_TS_Model_shader_name(void *component, const void *data);

void *ts_get_TS_Model_meshes(const void *component);
void *ts_get_TS_Model_numMeshes(const void *component);
void *ts_get_TS_Model_shader(const void *component);

#endif
