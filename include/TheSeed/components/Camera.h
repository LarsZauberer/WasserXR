#ifndef TS_CAMERA_H
#define TS_CAMERA_H

#include "TheSeed/ecs/Scene.h"

typedef struct TS_Camera TS_Camera;

void *ts_create_TS_Camera();
void ts_destroy_TS_Camera(void *cam);
void ts_schema_TS_Camera(TS_Component_Schema *schema);

void *ts_get_TS_Camera_fov(void *component);
void *ts_get_TS_Camera_near(void *component);
void *ts_get_TS_Camera_far(void *component);

void ts_set_TS_Camera_fov(void *component, void *data);
void ts_set_TS_Camera_near(void *component, void *data);
void ts_set_TS_Camera_far(void *component, void *data);

#endif
