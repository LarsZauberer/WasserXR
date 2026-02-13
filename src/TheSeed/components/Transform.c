#include "TheSeed/components/Transform.h"
#include "cglm/vec3.h"
#include <stdlib.h>

void *ts_create_TS_Transform() {
  TS_Transform_t *t = (TS_Transform_t *)malloc(sizeof(TS_Transform_t));

  glm_vec3_zero(t->position);
  glm_vec3_zero(t->rotation);
  glm_vec3_one(t->scale);

  // t->position[0] = -1.5f;
  //
  // t->rotation[0] = 45.0f;
  //
  // t->rotation[1] = 45.0f;

  return t;
}

void ts_destroy_TS_Transform(void *t) {
  free(t);
  return;
}
