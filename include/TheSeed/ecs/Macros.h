#ifndef TS_ECS_MACROS_H
#define TS_ECS_MACROS_H

#include <TheSeed/ecs/Scene.h>
#include <string.h>

#define TS_BASIC_SERIALIZE(component_type, field_name, field_exp, field_size)  \
  char *ts_serialize_##component_type##_##field_name(const void *ptr) {        \
    const component_type *component = ptr;                                     \
    char *field_id = #field_name;                                              \
                                                                               \
    size_t allocation = sizeof(size_t) + strlen(field_id) + 1 + field_size;    \
    char *data = (char *)malloc(allocation);                                   \
    char *iter = data;                                                         \
                                                                               \
    memcpy(iter, &allocation, sizeof(size_t));                                 \
    iter += sizeof(size_t);                                                    \
                                                                               \
    memcpy(iter, field_id, strlen(field_id) + 1);                              \
    iter += strlen(field_id) + 1;                                              \
                                                                               \
    memcpy(iter, field_exp, field_size);                                       \
    iter += field_size;                                                        \
                                                                               \
    return data;                                                               \
  }

#define TS_BASIC_DESERIALIZE(component_type, field_name, field_exp,            \
                             field_size)                                       \
  int ts_deserialize_##component_type##_##field_name(void *ptr,                \
                                                     const char *data) {       \
    component_type *component = ptr;                                           \
    memcpy(field_exp, data, field_size);                                       \
    return 0;                                                                  \
  }

#define TS_BASIC_SERIALIZERS(component_type, field_name, field_exp,            \
                             field_size)                                       \
  TS_BASIC_SERIALIZE(component_type, field_name, &component->field_exp,        \
                     field_size);                                              \
  TS_BASIC_DESERIALIZE(component_type, field_name, &component->field_exp,      \
                       field_size)

#define TS_STRING_SERIALIZE(component_type, field_name, field_exp)             \
  TS_BASIC_SERIALIZE(component_type, field_name, field_exp,                    \
                     strlen(field_exp) + 1)

#define TS_STRING_DESERIALIZE(component_type, field_name, field_exp)           \
  int ts_deserialize_##component_type##_##field_name(void *ptr,                \
                                                     const char *data) {       \
    component_type *component = ptr;                                           \
    field_exp = ts_copy_char_ptr(data);                                        \
    return 0;                                                                  \
  }

#define TS_STRING_SERIALIZERS(component_type, field_name, field_exp)           \
  TS_STRING_SERIALIZE(component_type, field_name, component->field_exp);       \
  TS_STRING_DESERIALIZE(component_type, field_name, component->field_exp)

#define TS_SET_DESERIALIZE(component_type, field_name, field_exp, setter)

#endif
