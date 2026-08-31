#include "wasserxr.h"

void *simple_creator(void) { return NULL; }

void simple_destroyer(void *ptr) {}

static const ComponentFieldDefinition fields[] = {
    {
        .name = "MyField",
        .getter = NULL,
        .mutable_ = 1,
        .serializer = NULL,
        .deserializer = NULL,
    },
};

static const ComponentDefinition components[] = {
    {
        .name = "MyComponent",
        .creator = simple_creator,
        .destroyer = simple_destroyer,
        .fields = fields,
        .field_count = 1,
    },
};

const PluginDefinition wxr_plugin = {
    .name = "MyPlugin",
    .engine_version = {0, 2, 0},
    .components = components,
    .component_count = 1,
};
