#ifndef TS_Transform_H
#define TS_Transform_H

#include "TheSeed/ecs/Scene.h"
typedef struct TS_Transform TS_Transform;

void *ts_create_TS_Transform();
void ts_destroy_TS_Transform(void *ptr);
void ts_schema_TS_Transform(TS_Component_Schema *schema);

#endif
