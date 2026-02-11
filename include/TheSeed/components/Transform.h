#ifndef TS_Transform_H
#define TS_Transform_H

#include "cglm/types.h"
typedef struct {
  vec3 position;
  vec3 rotation;
  vec3 scale;
} TS_Transform_t;

void *ts_create_TS_Transform();
void ts_destroy_TS_Transform(void *);

#endif
