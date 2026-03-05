#ifndef TS_Transform_H
#define TS_Transform_H

#include "TheSeed/ecs/Scene.h"
#include "cglm/types.h"
typedef struct {
  vec3 position;
  vec3 rotation;
  vec3 scale;
} TS_Transform;

void *ts_create_TS_Transform();
void ts_destroy_TS_Transform(void *ptr);
void ts_schema_TS_Transform(TS_Component_Schema *schema);
void *ts_get_TS_Transform_x(void *component);
void ts_set_TS_Transform_x(void *component, void *data);

#endif
