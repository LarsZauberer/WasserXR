#include "TheSeed/components/Transform.h"
#include "TheSeed/core/logging.h"
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

void ts_schema_TS_Transform(TS_Component_Schema *schema) {
  TS_Component_Field *field_x =
      ts_create_component_field("x", sizeof(float), TS_F, TS_Permission_All,
                                ts_get_TS_Transform_x, ts_set_TS_Transform_x);
  ts_add_field_to_component_schema(schema, field_x);

  TS_Component_Field *field_y =
      ts_create_component_field("y", sizeof(float), TS_F, TS_Permission_All,
                                ts_get_TS_Transform_y, ts_set_TS_Transform_y);
  ts_add_field_to_component_schema(schema, field_y);

  TS_Component_Field *field_z =
      ts_create_component_field("z", sizeof(float), TS_F, TS_Permission_All,
                                ts_get_TS_Transform_z, ts_set_TS_Transform_z);
  ts_add_field_to_component_schema(schema, field_z);

  TS_Component_Field *field_rx =
      ts_create_component_field("rx", sizeof(float), TS_F, TS_Permission_All,
                                ts_get_TS_Transform_rx, ts_set_TS_Transform_rx);
  ts_add_field_to_component_schema(schema, field_rx);

  TS_Component_Field *field_ry =
      ts_create_component_field("ry", sizeof(float), TS_F, TS_Permission_All,
                                ts_get_TS_Transform_ry, ts_set_TS_Transform_ry);
  ts_add_field_to_component_schema(schema, field_ry);

  TS_Component_Field *field_rz =
      ts_create_component_field("rz", sizeof(float), TS_F, TS_Permission_All,
                                ts_get_TS_Transform_rz, ts_set_TS_Transform_rz);
  ts_add_field_to_component_schema(schema, field_rz);

  TS_Component_Field *field_sx =
      ts_create_component_field("sx", sizeof(float), TS_F, TS_Permission_All,
                                ts_get_TS_Transform_sx, ts_set_TS_Transform_sx);
  ts_add_field_to_component_schema(schema, field_sx);

  TS_Component_Field *field_sy =
      ts_create_component_field("sy", sizeof(float), TS_F, TS_Permission_All,
                                ts_get_TS_Transform_sy, ts_set_TS_Transform_sy);
  ts_add_field_to_component_schema(schema, field_sy);

  TS_Component_Field *field_sz =
      ts_create_component_field("sz", sizeof(float), TS_F, TS_Permission_All,
                                ts_get_TS_Transform_sz, ts_set_TS_Transform_sz);
  ts_add_field_to_component_schema(schema, field_sz);
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

void *ts_get_TS_Transform_y(void *component) {
  TS_Transform *transform = (TS_Transform *)component;

  return &transform->position[1];
}

void ts_set_TS_Transform_y(void *component, void *data) {
  TS_Transform *transform = (TS_Transform *)component;
  if (data) {
    transform->position[1] = *(float *)data;
  }
}

void *ts_get_TS_Transform_z(void *component) {
  TS_Transform *transform = (TS_Transform *)component;

  return &transform->position[2];
}

void ts_set_TS_Transform_z(void *component, void *data) {
  TS_Transform *transform = (TS_Transform *)component;
  if (data) {
    transform->position[2] = *(float *)data;
  }
}

void *ts_get_TS_Transform_rx(void *component) {
  TS_Transform *transform = (TS_Transform *)component;

  return &transform->rotation[0];
}

void ts_set_TS_Transform_rx(void *component, void *data) {
  TS_Transform *transform = (TS_Transform *)component;
  if (data) {
    transform->rotation[0] = *(float *)data;
  }
}

void *ts_get_TS_Transform_ry(void *component) {
  TS_Transform *transform = (TS_Transform *)component;

  return &transform->rotation[1];
}

void ts_set_TS_Transform_ry(void *component, void *data) {
  TS_Transform *transform = (TS_Transform *)component;
  if (data) {
    transform->rotation[1] = *(float *)data;
  }
}

void *ts_get_TS_Transform_rz(void *component) {
  TS_Transform *transform = (TS_Transform *)component;

  return &transform->rotation[2];
}

void ts_set_TS_Transform_rz(void *component, void *data) {
  TS_Transform *transform = (TS_Transform *)component;
  if (data) {
    transform->rotation[2] = *(float *)data;
  }
}

void *ts_get_TS_Transform_sx(void *component) {
  TS_Transform *transform = (TS_Transform *)component;

  return &transform->scale[0];
}

void ts_set_TS_Transform_sx(void *component, void *data) {
  TS_Transform *transform = (TS_Transform *)component;
  if (data) {
    transform->scale[0] = *(float *)data;
  }
}

void *ts_get_TS_Transform_sy(void *component) {
  TS_Transform *transform = (TS_Transform *)component;

  return &transform->scale[1];
}

void ts_set_TS_Transform_sy(void *component, void *data) {
  TS_Transform *transform = (TS_Transform *)component;
  if (data) {
    transform->scale[1] = *(float *)data;
  }
}

void *ts_get_TS_Transform_sz(void *component) {
  TS_Transform *transform = (TS_Transform *)component;

  return &transform->scale[2];
}

void ts_set_TS_Transform_sz(void *component, void *data) {
  TS_Transform *transform = (TS_Transform *)component;
  if (data) {
    transform->scale[2] = *(float *)data;
  }
}
