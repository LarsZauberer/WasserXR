#include "TheSeed/components/Camera.h"
#include "TheSeed/core/logging.h"
#include "TheSeed/ecs/Scene.h"
#include <stdlib.h>

void *ts_create_TS_Camera() {
  TS_Camera *cam = (TS_Camera *)malloc(sizeof(TS_Camera));
  ts_assert(cam, "Malloc failed during ts_create_TS_Camera");

  cam->fov = 90.0F;
  cam->near = 0.1F;
  cam->far = 100.0F;

  return cam;
}

void ts_destroy_TS_Camera(void *cam) { free(cam); }

void ts_schema_TS_Camera(TS_Component_Schema *schema) {}
