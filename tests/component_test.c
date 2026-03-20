#include "TheSeed/core/utils.h"
#include "TheSeed/ecs/Macros.h"
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

TS_BASIC_SERIALIZE(TS_A, x, &component->x, sizeof(int));
TS_BASIC_DESERIALIZE(TS_A, x, &component->x, sizeof(int));
TS_BASIC_SERIALIZE(TS_A, extra, &component->extra, sizeof(int));
TS_BASIC_DESERIALIZE(TS_A, extra, &component->extra, sizeof(int));

void ts_schema_TS_A(TS_Component_Schema *schema) {
  TS_Component_Field *field_x = ts_create_component_field(
      "x", sizeof(int), TS_L, ts_get_TS_A_x, ts_set_TS_A_x, ts_serialize_TS_A_x,
      ts_deserialize_TS_A_x);
  TS_Component_Field *field_extra = ts_create_component_field(
      "extra", sizeof(int), TS_L, ts_get_TS_A_extra, ts_set_TS_A_extra,
      ts_serialize_TS_A_extra, ts_deserialize_TS_A_extra);

  ts_add_field_to_component_schema(schema, field_x);
  ts_add_field_to_component_schema(schema, field_extra);
}

typedef struct TS_B TS_B;

struct TS_B {
  char *name;
};

void *ts_create_TS_B() {
  TS_B *component = (TS_B *)malloc(sizeof(TS_B));
  component->name = ts_copy_char_ptr("Hello World!");
  return component;
}

void ts_destroy_TS_B(void *ptr) {
  TS_B *component = ptr;
  free(component->name);
  free(component);
}

TS_STRING_SERIALIZERS(TS_B, name, component->name);

void *ts_get_TS_B_name(void *ptr) {
  TS_B *component = ptr;
  return component->name;
}

void ts_set_TS_B_name(void *ptr, void *value) {
  TS_B *component = ptr;
  component->name = value;
}

void ts_schema_TS_B(TS_Component_Schema *schema) {
  TS_Component_Field *field_name = ts_create_component_field(
      "name", sizeof(char *), TS_S, ts_get_TS_B_name, ts_set_TS_B_name,
      ts_serialize_TS_B_name, ts_deserialize_TS_B_name);

  ts_add_field_to_component_schema(schema, field_name);
}
