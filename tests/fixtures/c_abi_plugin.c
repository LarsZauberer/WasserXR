#include <stdint.h>
#include <stdlib.h>
#include <wasserxr.h>

typedef struct AbiCounter {
  int32_t value;
} AbiCounter;

static void *abi_counter_value(void *data) {
  return &((AbiCounter *)data)->value;
}

void *wxr_create_abi_counter(WXRScene *scene) {
  (void)scene;
  AbiCounter *counter = malloc(sizeof(AbiCounter));
  if (counter == NULL) {
    return NULL;
  }
  counter->value = 0;
  return counter;
}

void wxr_destroy_abi_counter(void *data) { free(data); }

void wxr_schema_abi_counter(WXRComponentSchema *schema) {
  wxr_component_schema_add_field(schema, "value", FieldType_I32,
                                 abi_counter_value, 1);
}

size_t WXR_GROUPS_ABI_COUNTER_SYSTEM = 1;

int32_t wxr_select_abi_counter_system(const WXRScene *scene, WXREntity entity) {
  return wxr_has_component(scene, entity, "abi_counter") == 1 ? 0 : -1;
}

void wxr_system_abi_counter_system(WXRScene *scene,
                                   const WXREntity *const *entities,
                                   const size_t *groups) {
  for (size_t i = 0; i < groups[0]; i++) {
    int32_t *value = wxr_query(scene, entities[0][i], "abi_counter", "value");
    if (value != NULL) {
      *value += 1;
    }
  }
}
