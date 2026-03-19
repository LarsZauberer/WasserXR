#include "TheSeed/ecs/Macros.h"
#include "TheSeed/ecs/Scene.h"
#include <stdlib.h>
#include <string.h>
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

char *ts_serialize_TS_A_x(const void *ptr) {
  const TS_A *component = ptr;
  char *field_id = "x";
  size_t allocation = sizeof(size_t) + strlen(field_id) + 1 + sizeof(int);
  char *data = (char *)malloc(allocation);
  char *iter = data;
  memcpy(iter, &allocation, sizeof(size_t));
  iter += sizeof(size_t);
  memcpy(iter, field_id, strlen(field_id) + 1);
  iter += strlen(field_id) + 1;
  memcpy(iter, &component->x, sizeof(int));
  iter += sizeof(int);
  return data;
}

int ts_deserialize_TS_A_x(void *ptr, const char *data) {
  TS_A *component = ptr;
  memcpy(&component->x, data, sizeof(int));
  return 0;
}

TS_BASIC_SERIALIZE(TS_A, extra, int, "extra")

int ts_deserialize_TS_A_extra(void *ptr, const char *data) {
  TS_A *component = ptr;
  memcpy(&component->extra, data, sizeof(int));
  return 0;
}

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
