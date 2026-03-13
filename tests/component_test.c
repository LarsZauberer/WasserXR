#include "TheSeed/ecs/Scene.h"
#include <stdlib.h>
typedef struct TS_A TS_A;

struct TS_A {
  int x;
  int extra;
};

void *ts_create_TS_A() {
  TS_A *data = malloc(sizeof(TS_A));

  data->x = 1;
  data->extra = 5;

  return data;
}

void ts_destroy_TS_A(void *component) { free(component); }

void *ts_get_TS_A_x(void *ptr) {
  TS_A *component = ptr;
  return &component->x;
}

void ts_set_TS_A_x(void *ptr, void *value) {
  TS_A *component = ptr;
  int val = *(int *)value;
  component->x = val;
}

void *ts_get_TS_A_extra(void *ptr) {
  TS_A *component = ptr;
  return &component->extra;
}

void ts_set_TS_A_extra(void *ptr, void *value) {
  TS_A *component = ptr;
  int val = *(int *)value;
  component->extra = val;
}

void ts_schema_TS_A(TS_Component_Schema *schema) {
  TS_Component_Field *field_x = ts_create_component_field(
      "x", sizeof(int), TS_L, TS_Permission_All, ts_get_TS_A_x, ts_set_TS_A_x);
  TS_Component_Field *field_extra =
      ts_create_component_field("extra", sizeof(int), TS_L, TS_Permission_All,
                                ts_get_TS_A_extra, ts_set_TS_A_extra);

  ts_add_field_to_component_schema(schema, field_x);
  ts_add_field_to_component_schema(schema, field_extra);
}
