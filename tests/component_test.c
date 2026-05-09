#include "WasserXR/ecs/utils.h"
#include "WasserXR/ecs/Macros.h"
#include "WasserXR/ecs/Scene.h"
#include <stdlib.h>

typedef struct WXR_A WXR_A;

struct WXR_A {
  int x;
  int extra;
};

void *wxr_create_WXR_A() {
  WXR_A *data = malloc(sizeof(WXR_A));

  data->x = 1;
  data->extra = 5;

  return data;
}

void wxr_destroy_WXR_A(void *component) { free(component); }

WXR_BASIC_GETTER(WXR_A, x, &component->x, sizeof(int));
WXR_BASIC_SETTER(WXR_A, x, &component->x, sizeof(int));

WXR_BASIC_GETTER(WXR_A, extra, &component->extra, sizeof(int));
WXR_BASIC_SETTER(WXR_A, extra, &component->extra, sizeof(int));

WXR_BASIC_SERIALIZE(WXR_A, x, &component->x, sizeof(int));
WXR_BASIC_DESERIALIZE(WXR_A, x, &component->x, sizeof(int));
WXR_BASIC_SERIALIZE(WXR_A, extra, &component->extra, sizeof(int));
WXR_BASIC_DESERIALIZE(WXR_A, extra, &component->extra, sizeof(int));

void wxr_schema_WXR_A(WXR_Component_Schema *schema) {
  WXR_SCHEMA_FIELD(WXR_L, x, wxr_get_WXR_A_x, wxr_set_WXR_A_x, wxr_serialize_WXR_A_x,
                  wxr_deserialize_WXR_A_x);
  WXR_SCHEMA_FIELD(WXR_L, extra, wxr_get_WXR_A_extra, wxr_set_WXR_A_extra,
                  wxr_serialize_WXR_A_extra, wxr_deserialize_WXR_A_extra);
}

typedef struct WXR_B WXR_B;

struct WXR_B {
  char *name;
};

void *wxr_create_WXR_B() {
  WXR_B *component = (WXR_B *)malloc(sizeof(WXR_B));
  component->name = wxr_copy_char_ptr("Hello World!");
  return component;
}

void wxr_destroy_WXR_B(void *ptr) {
  WXR_B *component = ptr;
  free(component->name);
  free(component);
}

WXR_STRING_SERIALIZERS(WXR_B, name, component->name);

WXR_STRING_GETTER(WXR_B, name, component->name);
WXR_STRING_SETTER(WXR_B, name, component->name);

void wxr_schema_WXR_B(WXR_Component_Schema *schema) {
  WXR_SCHEMA_FIELD(WXR_S, name, wxr_get_WXR_B_name, wxr_set_WXR_B_name,
                  wxr_serialize_WXR_B_name, wxr_deserialize_WXR_B_name);
}

typedef struct WXR_C_Empty WXR_C_Empty;

struct WXR_C_Empty {
  int a;
};

void *wxr_create_WXR_C_Empty() {
  WXR_C_Empty *component = malloc(sizeof(WXR_C));
  component->a = 5;
  return component;
}

void wxr_destroy_WXR_C_Empty(void *ptr) { free(ptr); }

void wxr_schema_WXR_C_Empty(WXR_Component_Schema *schema) {}
