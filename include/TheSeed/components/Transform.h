#ifndef TS_Transform_H
#define TS_Transform_H

#include "TheSeed/ecs/Scene.h"
typedef struct TS_Transform TS_Transform;

void *ts_create_TS_Transform();
void ts_destroy_TS_Transform(void *ptr);
void ts_schema_TS_Transform(TS_Component_Schema *schema);
void *ts_get_TS_Transform_x(void *component);
void ts_set_TS_Transform_x(void *component, void *data);
void *ts_get_TS_Transform_y(void *component);
void ts_set_TS_Transform_y(void *component, void *data);
void *ts_get_TS_Transform_z(void *component);
void ts_set_TS_Transform_z(void *component, void *data);
void *ts_get_TS_Transform_rx(void *component);
void ts_set_TS_Transform_rx(void *component, void *data);
void *ts_get_TS_Transform_ry(void *component);
void ts_set_TS_Transform_ry(void *component, void *data);
void *ts_get_TS_Transform_rz(void *component);
void ts_set_TS_Transform_rz(void *component, void *data);
void *ts_get_TS_Transform_sx(void *component);
void ts_set_TS_Transform_sx(void *component, void *data);
void *ts_get_TS_Transform_sy(void *component);
void ts_set_TS_Transform_sy(void *component, void *data);
void *ts_get_TS_Transform_sz(void *component);
void ts_set_TS_Transform_sz(void *component, void *data);

#endif
