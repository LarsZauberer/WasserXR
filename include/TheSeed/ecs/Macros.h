#ifndef TS_MACROS_H
#define TS_MACROS_H

#include <TheSeed/ecs/Scene.h>

#define TS_BASIC_SERIALIZE(component_type, field, field_type, field_name)      \
  char *ts_serialize_##component_type##_##field##(const void *ptr) {           \
    const component_type *component = ptr;                                     \
    char *field_id = field_name;                                               \
    size_t allocation =                                                        \
        sizeof(size_t) + strlen(field_id) + 1 + sizeof(field_type);            \
    char *data = (char *)malloc(allocation);                                   \
    char *iter = data;                                                         \
    memcpy(iter, &allocation, sizeof(size_t));                                 \
    iter += sizeof(size_t);                                                    \
    memcpy(iter, field_id, strlen(field_id) + 1);                              \
    iter += strlen(field_id) + 1;                                              \
    memcpy(iter, &component->##field##, sizeof(field_type));                   \
    iter += sizeof(field_type);                                                \
    return data;                                                               \
  }

#endif
