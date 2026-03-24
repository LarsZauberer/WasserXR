#include "TheSeed/components/Camera.h"
#include "TheSeed/core/logging.h"
#include "TheSeed/ecs/Macros.h"
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

TS_BASIC_SERIALIZERS(TS_Camera, fov, &component->fov, sizeof(float));
TS_BASIC_SERIALIZERS(TS_Camera, near, &component->near, sizeof(float));
TS_BASIC_SERIALIZERS(TS_Camera, far, &component->far, sizeof(float));

void ts_destroy_TS_Camera(void *cam) { free(cam); }

void ts_schema_TS_Camera(TS_Component_Schema *schema) {
  TS_Component_Field *fov_field = ts_create_component_field(
      "fov", sizeof(float), TS_F, ts_get_TS_Camera_fov, ts_set_TS_Camera_fov,
      ts_serialize_TS_Camera_fov, ts_deserialize_TS_Camera_fov);
  TS_Component_Field *near_field = ts_create_component_field(
      "near", sizeof(float), TS_F, ts_get_TS_Camera_near, ts_set_TS_Camera_near,
      ts_serialize_TS_Camera_near, ts_deserialize_TS_Camera_near);
  TS_Component_Field *far_field = ts_create_component_field(
      "far", sizeof(float), TS_F, ts_get_TS_Camera_far, ts_set_TS_Camera_far,
      ts_serialize_TS_Camera_far, ts_deserialize_TS_Camera_far);

  ts_add_field_to_component_schema(schema, fov_field);
  ts_add_field_to_component_schema(schema, near_field);
  ts_add_field_to_component_schema(schema, far_field);
}

void *ts_get_TS_Camera_fov(const void *component) {
  const TS_Camera *cam = (const TS_Camera *)component;
  return (void *)&cam->fov;
}

void *ts_get_TS_Camera_near(const void *component) {
  const TS_Camera *cam = (const TS_Camera *)component;
  return (void *)&cam->near;
}

void *ts_get_TS_Camera_far(const void *component) {
  const TS_Camera *cam = (const TS_Camera *)component;
  return (void *)&cam->far;
}

void ts_set_TS_Camera_fov(void *component, const void *data) {
  if (!data) {
    ts_warn("Trying to set NULL data to TS_Camera");
    return;
  }
  TS_Camera *cam = (TS_Camera *)component;
  float fov = *(const float *)data;
  cam->fov = fov;
}

void ts_set_TS_Camera_near(void *component, const void *data) {
  if (!data) {
    ts_warn("Trying to set NULL data to TS_Camera");
    return;
  }
  TS_Camera *cam = (TS_Camera *)component;
  float near = *(const float *)data;
  cam->near = near;
}

void ts_set_TS_Camera_far(void *component, const void *data) {
  if (!data) {
    ts_warn("Trying to set NULL data to TS_Camera");
    return;
  }
  TS_Camera *cam = (TS_Camera *)component;
  float far = *(const float *)data;
  cam->far = far;
}
