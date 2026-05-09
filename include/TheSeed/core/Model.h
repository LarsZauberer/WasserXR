#ifndef TS_MODEL_H
#define TS_MODEL_H

#include "TheSeed/ecs/Scene.h"

typedef struct TS_Model TS_Model;

void *ts_create_TS_Model();
void ts_destroy_TS_Model(void *ptr);
void ts_schema_TS_Model(TS_Component_Schema *schema);

#endif
