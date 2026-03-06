#include "TheSeed/components/Camera.h"
#include "TheSeed/core/logging.h"
#include "TheSeed/ecs/Scene.h"
#include <stdlib.h>

struct TS_Camera {
  float fov;
  float near;
  float far;
};

void *ts_create_TS_Camera() {
  TS_Camera *cam = (TS_Camera *)malloc(sizeof(TS_Camera));
  ts_assert(cam, "Malloc failed during ts_create_TS_Camera");

  cam->fov = 90.0F;
  cam->near = 0.1F;
  cam->far = 100.0F;

  return cam;
}

void ts_destroy_TS_Camera(void *cam) { free(cam); }

void ts_schema_TS_Camera(TS_Component_Schema *schema) {
  TS_Component_Field *fov_field =
      ts_create_component_field("fov", sizeof(float), TS_F, TS_Permission_All,
                                ts_get_TS_Camera_fov, ts_set_TS_Camera_fov);
  TS_Component_Field *near_field =
      ts_create_component_field("near", sizeof(float), TS_F, TS_Permission_All,
                                ts_get_TS_Camera_near, ts_set_TS_Camera_near);
  TS_Component_Field *far_field =
      ts_create_component_field("far", sizeof(float), TS_F, TS_Permission_All,
                                ts_get_TS_Camera_far, ts_set_TS_Camera_far);

  ts_add_field_to_component_schema(schema, fov_field);
  ts_add_field_to_component_schema(schema, near_field);
  ts_add_field_to_component_schema(schema, far_field);
}

void *ts_get_TS_Camera_fov(void *component) {
  TS_Camera *cam = (TS_Camera *)component;
  return &cam->fov;
}

void *ts_get_TS_Camera_near(void *component) {
  TS_Camera *cam = (TS_Camera *)component;
  return &cam->near;
}

void *ts_get_TS_Camera_far(void *component) {
  TS_Camera *cam = (TS_Camera *)component;
  return &cam->far;
}

void ts_set_TS_Camera_fov(void *component, void *data) {
  if (!data) {
    ts_warn("Trying to set NULL data to TS_Camera");
    return;
  }
  TS_Camera *cam = (TS_Camera *)component;
  float fov = *(float *)data;
  cam->fov = fov;
}

void ts_set_TS_Camera_near(void *component, void *data) {
  if (!data) {
    ts_warn("Trying to set NULL data to TS_Camera");
    return;
  }
  TS_Camera *cam = (TS_Camera *)component;
  float near = *(float *)data;
  cam->near = near;
}

void ts_set_TS_Camera_far(void *component, void *data) {
  if (!data) {
    ts_warn("Trying to set NULL data to TS_Camera");
    return;
  }
  TS_Camera *cam = (TS_Camera *)component;
  float far = *(float *)data;
  cam->far = far;
}
