#include "TheSeed/components/Camera.h"
#include "TheSeed/ecs/logging.h"
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

TS_BASIC_ACCESS(TS_Camera, fov, &component->fov, sizeof(float));
TS_BASIC_ACCESS(TS_Camera, near, &component->near, sizeof(float));
TS_BASIC_ACCESS(TS_Camera, far, &component->far, sizeof(float));

void ts_destroy_TS_Camera(void *cam) { free(cam); }

void ts_schema_TS_Camera(TS_Component_Schema *schema) {
  TS_SCHEMA_FIELD_FULL(TS_Camera, TS_F, fov);
  TS_SCHEMA_FIELD_FULL(TS_Camera, TS_F, near);
  TS_SCHEMA_FIELD_FULL(TS_Camera, TS_F, far);
}
