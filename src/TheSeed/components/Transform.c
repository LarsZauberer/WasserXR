#include "TheSeed/components/Transform.h"
#include "TheSeed/core/logging.h"
#include "TheSeed/ecs/Macros.h"
#include "TheSeed/ecs/Scene.h"
#include "cglm/vec3.h"
#include <stdlib.h>

struct TS_Transform {
  vec3 position;
  vec3 rotation;
  vec3 scale;
};

void *ts_create_TS_Transform() {
  TS_Transform *ptr = (TS_Transform *)malloc(sizeof(TS_Transform));
  ts_assert_abort_value(ptr, NULL,
                        "Malloc failed during ts_create_TS_Transform");

  glm_vec3_zero(ptr->position);
  glm_vec3_zero(ptr->rotation);
  glm_vec3_one(ptr->scale);

  return ptr;
}

void ts_destroy_TS_Transform(void *ptr) { free(ptr); }

TS_BASIC_SERIALIZERS(TS_Transform, x, &component->position[0], sizeof(float));
TS_BASIC_SERIALIZERS(TS_Transform, y, &component->position[1], sizeof(float));
TS_BASIC_SERIALIZERS(TS_Transform, z, &component->position[2], sizeof(float));
TS_BASIC_SERIALIZERS(TS_Transform, rx, &component->rotation[0], sizeof(float));
TS_BASIC_SERIALIZERS(TS_Transform, ry, &component->rotation[1], sizeof(float));
TS_BASIC_SERIALIZERS(TS_Transform, rz, &component->rotation[2], sizeof(float));
TS_BASIC_SERIALIZERS(TS_Transform, sx, &component->scale[0], sizeof(float));
TS_BASIC_SERIALIZERS(TS_Transform, sy, &component->scale[1], sizeof(float));
TS_BASIC_SERIALIZERS(TS_Transform, sz, &component->scale[2], sizeof(float));

TS_BASIC_ACCESS(TS_Transform, x, &component->position[0], sizeof(float));
TS_BASIC_ACCESS(TS_Transform, y, &component->position[1], sizeof(float));
TS_BASIC_ACCESS(TS_Transform, z, &component->position[2], sizeof(float));
TS_BASIC_ACCESS(TS_Transform, rx, &component->rotation[0], sizeof(float));
TS_BASIC_ACCESS(TS_Transform, ry, &component->rotation[1], sizeof(float));
TS_BASIC_ACCESS(TS_Transform, rz, &component->rotation[2], sizeof(float));
TS_BASIC_ACCESS(TS_Transform, sx, &component->scale[0], sizeof(float));
TS_BASIC_ACCESS(TS_Transform, sy, &component->scale[1], sizeof(float));
TS_BASIC_ACCESS(TS_Transform, sz, &component->scale[2], sizeof(float));

void ts_schema_TS_Transform(TS_Component_Schema *schema) {
  TS_SCHEMA_FIELD_FULL(TS_Transform, TS_F, x);
  TS_SCHEMA_FIELD_FULL(TS_Transform, TS_F, y);
  TS_SCHEMA_FIELD_FULL(TS_Transform, TS_F, z);
  TS_SCHEMA_FIELD_FULL(TS_Transform, TS_F, rx);
  TS_SCHEMA_FIELD_FULL(TS_Transform, TS_F, ry);
  TS_SCHEMA_FIELD_FULL(TS_Transform, TS_F, rz);
  TS_SCHEMA_FIELD_FULL(TS_Transform, TS_F, sx);
  TS_SCHEMA_FIELD_FULL(TS_Transform, TS_F, sy);
  TS_SCHEMA_FIELD_FULL(TS_Transform, TS_F, sz);
}
