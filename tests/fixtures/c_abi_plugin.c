#include <stdint.h>
#include <stdlib.h>
#include <wasserxr.h>

typedef struct AbiCounter {
  int32_t value;
} AbiCounter;

static void *abi_counter_value(void *data) {
  return &((AbiCounter *)data)->value;
}

static void *abi_counter_create(Scene *scene) {
  (void)scene;
  return calloc(1, sizeof(AbiCounter));
}

static void abi_counter_destroy(void *data) { free(data); }

static void abi_counter_system(Scene *scene, float delta,
                               const WXREntity *const *entities,
                               const size_t *entity_counts,
                               size_t entity_group_count) {
  (void)delta;
  if (entity_group_count != 1) {
    return;
  }

  for (size_t i = 0; i < entity_counts[0]; i++) {
    int32_t *value = wxr_query((WXRScene *)scene, entities[0][i], "abi_counter",
                               "value");
    if (value != NULL) {
      *value += 1;
    }
  }
}

static const WXRComponentFieldDescriptor ABI_COUNTER_FIELDS[] = {{
    .name = "value",
    .field_type = FieldType_I32,
    .getter = abi_counter_value,
    .mutable_ = 1,
    .serializer = NULL,
    .deserializer = NULL,
}};

static const WXRComponentDescriptor ABI_COMPONENTS[] = {{
    .name = "abi_counter",
    .creator = abi_counter_create,
    .destroyer = abi_counter_destroy,
    .fields = ABI_COUNTER_FIELDS,
    .field_count = 1,
    .methods = NULL,
    .method_count = 0,
}};

static const char *const ABI_COUNTER_COMPONENTS[] = {"abi_counter"};
static const WXRSystemEntityGroupDescriptor ABI_COUNTER_GROUPS[] = {{
    .components = ABI_COUNTER_COMPONENTS,
    .component_count = 1,
}};
static const WXRSystemDescriptor ABI_SYSTEMS[] = {{
    .name = "abi_counter_system",
    .runner = abi_counter_system,
    .attach = NULL,
    .detach = NULL,
    .entity_groups = ABI_COUNTER_GROUPS,
    .entity_group_count = 1,
}};

const WXRPluginDescriptor wxr_plugin = {
    .version = {
        .major = WXR_VERSION_MAJOR,
        .minor = WXR_VERSION_MINOR,
        .patch = WXR_VERSION_PATCH,
    },
    .name = "c-abi-test",
    .components = ABI_COMPONENTS,
    .component_count = 1,
    .assets = NULL,
    .asset_count = 0,
    .systems = ABI_SYSTEMS,
    .system_count = 1,
};
