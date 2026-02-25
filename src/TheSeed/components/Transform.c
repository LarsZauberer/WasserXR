#include "TheSeed/components/Transform.h"
#include "TheSeed/core/logging.h"
#include "cglm/types.h"
#include "cglm/vec3.h"
#include <stdlib.h>

void *ts_create_TS_Transform() {
  TS_Transform *ptr = (TS_Transform *)malloc(sizeof(TS_Transform));
  ts_assert(ptr, "Malloc failed during ts_create_TS_Transform");

  glm_vec3_zero(ptr->position);
  glm_vec3_zero(ptr->rotation);
  glm_vec3_one(ptr->scale);

  return ptr;
}

void ts_destroy_TS_Transform(void *ptr) { free(ptr); }

void ts_serialize_TS_Transform(void *ptr, TS_Serialization *serialization) {
  TS_Transform *transform = (TS_Transform *)ptr;

  vec3 *position = (vec3 *)malloc(sizeof(vec3));
  vec3 *rotation = (vec3 *)malloc(sizeof(vec3));
  vec3 *scale = (vec3 *)malloc(sizeof(vec3));

  (*position)[0] = transform->position[0];
  (*position)[1] = transform->position[1];
  (*position)[2] = transform->position[2];

  (*rotation)[0] = transform->rotation[0];
  (*rotation)[1] = transform->rotation[1];
  (*rotation)[2] = transform->rotation[2];

  (*scale)[0] = transform->scale[0];
  (*scale)[1] = transform->scale[1];
  (*scale)[2] = transform->scale[2];

  ts_set_serialization(serialization, "position", sizeof(vec3), position);
  ts_set_serialization(serialization, "rotation", sizeof(vec3), rotation);
  ts_set_serialization(serialization, "scale", sizeof(vec3), scale);
}

void ts_deserialize_TS_Transform(void *ptr, TS_Serialization *serialization) {
  TS_Transform *transform = (TS_Transform *)ptr;

  vec3 *position = (vec3 *)ts_get_serialization(serialization, "position");
  vec3 *rotation = (vec3 *)ts_get_serialization(serialization, "rotation");
  vec3 *scale = (vec3 *)ts_get_serialization(serialization, "scale");

  if (position) {
    transform->position[0] = (*position)[0];
    transform->position[1] = (*position)[1];
    transform->position[2] = (*position)[2];
  }

  if (rotation) {
    transform->rotation[0] = (*rotation)[0];
    transform->rotation[1] = (*rotation)[1];
    transform->rotation[2] = (*rotation)[2];
  }

  if (scale) {
    transform->scale[0] = (*scale)[0];
    transform->scale[1] = (*scale)[1];
    transform->scale[2] = (*scale)[2];
  }
}
