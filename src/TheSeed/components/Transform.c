#include "TheSeed/components/Transform.h"
#include "TheSeed/core/logging.h"
#include "TheSeed/ecs/Scene.h"
#include "cglm/vec3.h"
#include <stdlib.h>

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

void ts_schema_TS_Transform(TS_Component_Schema *schema) {
  TS_Component_Field *field_x =
      ts_create_component_field("x", sizeof(float), TS_F, TS_Permission_All,
                                ts_get_TS_Transform_x, ts_set_TS_Transform_x);
  ts_add_field_to_component_schema(schema, field_x);
}

void *ts_get_TS_Transform_x(void *component) {
  TS_Transform *transform = (TS_Transform *)component;

  return &transform->position[0];
}

void ts_set_TS_Transform_x(void *component, void *data) {
  TS_Transform *transform = (TS_Transform *)component;
  if (data) {
    transform->position[0] = *(float *)data;
  }
}
