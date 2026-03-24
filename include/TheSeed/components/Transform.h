#ifndef TS_Transform_H
#define TS_Transform_H

#include "TheSeed/ecs/Scene.h"
typedef struct TS_Transform TS_Transform;

void *ts_create_TS_Transform();
void ts_destroy_TS_Transform(void *ptr);
void ts_schema_TS_Transform(TS_Component_Schema *schema);
void *ts_get_TS_Transform_x(const void *component);
void ts_set_TS_Transform_x(void *component, const void *data);
void *ts_get_TS_Transform_y(const void *component);
void ts_set_TS_Transform_y(void *component, const void *data);
void *ts_get_TS_Transform_z(const void *component);
void ts_set_TS_Transform_z(void *component, const void *data);
void *ts_get_TS_Transform_rx(const void *component);
void ts_set_TS_Transform_rx(void *component, const void *data);
void *ts_get_TS_Transform_ry(const void *component);
void ts_set_TS_Transform_ry(void *component, const void *data);
void *ts_get_TS_Transform_rz(const void *component);
void ts_set_TS_Transform_rz(void *component, const void *data);
void *ts_get_TS_Transform_sx(const void *component);
void ts_set_TS_Transform_sx(void *component, const void *data);
void *ts_get_TS_Transform_sy(const void *component);
void ts_set_TS_Transform_sy(void *component, const void *data);
void *ts_get_TS_Transform_sz(const void *component);
void ts_set_TS_Transform_sz(void *component, const void *data);

#endif
