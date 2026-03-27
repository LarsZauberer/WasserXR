#ifndef TS_CAMERA_H
#define TS_CAMERA_H

#include "TheSeed/ecs/Scene.h"

typedef struct TS_Camera TS_Camera;

void *ts_create_TS_Camera();
void ts_destroy_TS_Camera(void *cam);
void ts_schema_TS_Camera(TS_Component_Schema *schema);

#endif
