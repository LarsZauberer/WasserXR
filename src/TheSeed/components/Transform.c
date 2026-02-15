#include "TheSeed/components/Transform.h"
#include "cglm/types.h"
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

void ts_serialize_TS_Transform(void *t, TS_Serialization *serialization) {
  TS_Transform_t *transform = (TS_Transform_t *)t;

  vec3 *position = (vec3 *)malloc(sizeof(vec3));

  (*position)[0] = transform->position[0];
  (*position)[1] = transform->position[1];
  (*position)[2] = transform->position[2];

  ts_set_serialization(serialization, "position", sizeof(vec3), position);
  return;
}

void ts_deserialize_TS_Transform(void *t, TS_Serialization *serialization) {
  TS_Transform_t *transform = (TS_Transform_t *)t;

  vec3 *position = (vec3 *)ts_get_serialization(serialization, "position");
  transform->position[0] = (*position)[0];
  transform->position[1] = (*position)[1];
  transform->position[2] = (*position)[2];
  return;
}
