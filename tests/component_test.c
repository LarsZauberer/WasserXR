#include "TheSeed/ecs/utils.h"
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

TS_BASIC_GETTER(TS_A, x, &component->x, sizeof(int));
TS_BASIC_SETTER(TS_A, x, &component->x, sizeof(int));

TS_BASIC_GETTER(TS_A, extra, &component->extra, sizeof(int));
TS_BASIC_SETTER(TS_A, extra, &component->extra, sizeof(int));

TS_BASIC_SERIALIZE(TS_A, x, &component->x, sizeof(int));
TS_BASIC_DESERIALIZE(TS_A, x, &component->x, sizeof(int));
TS_BASIC_SERIALIZE(TS_A, extra, &component->extra, sizeof(int));
TS_BASIC_DESERIALIZE(TS_A, extra, &component->extra, sizeof(int));

void ts_schema_TS_A(TS_Component_Schema *schema) {
  TS_SCHEMA_FIELD(TS_L, x, ts_get_TS_A_x, ts_set_TS_A_x, ts_serialize_TS_A_x,
                  ts_deserialize_TS_A_x);
  TS_SCHEMA_FIELD(TS_L, extra, ts_get_TS_A_extra, ts_set_TS_A_extra,
                  ts_serialize_TS_A_extra, ts_deserialize_TS_A_extra);
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

TS_STRING_GETTER(TS_B, name, component->name);
TS_STRING_SETTER(TS_B, name, component->name);

void ts_schema_TS_B(TS_Component_Schema *schema) {
  TS_SCHEMA_FIELD(TS_S, name, ts_get_TS_B_name, ts_set_TS_B_name,
                  ts_serialize_TS_B_name, ts_deserialize_TS_B_name);
}

typedef struct TS_C_Empty TS_C_Empty;

struct TS_C_Empty {
  int a;
};

void *ts_create_TS_C_Empty() {
  TS_C_Empty *component = malloc(sizeof(TS_C));
  component->a = 5;
  return component;
}

void ts_destroy_TS_C_Empty(void *ptr) { free(ptr); }

void ts_schema_TS_C_Empty(TS_Component_Schema *schema) {}
